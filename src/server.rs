use std::net::{SocketAddr, TcpListener, TcpStream};
use crate::leader::{IS_LEADER, do_leader_workload};
use crate::message::{init_message_queue, RequestVotePayload, RequestVoteMessage, calculate_hash, MessageType, generate_request_vote_payload};
use crate::log::{initialize_raft_log, TheLog, get_raft_log};
use std::collections::VecDeque;
use crate::kv_store::{set_key, get_key};
use std::{thread, time};
use crate::connection_handler::{connection_handler, serialize_request_vote, handle_resp};
use serde::{Serialize, Deserialize};
use std::io::Write;

struct Addrs {
    addresses: [SocketAddr; 5],
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct RaftClusterPeers {
    pub addresses: VecDeque<SocketAddr>,
}

fn set_raft_cluster_peers(peers: Addrs, tcp_listener: TcpListener) {
    //get RaftClusterPeers
    let mut cluster_peers = RaftClusterPeers{
        addresses: VecDeque::new()
    };
    for address in peers.addresses.iter() {

        if address.port() != tcp_listener.local_addr().unwrap().port() {
            cluster_peers.addresses.push_back(*address)
        }
    }

    let serialized_peers = serde_json::to_string(&cluster_peers).unwrap();
    set_key("peers".parse().unwrap(), serialized_peers);
    //print out followers
    let peers: RaftClusterPeers = serde_json::from_str(&get_key(String::from("peers"))).unwrap();
    println!("{:#?}", peers);
}

//getter for RaftClusterPeers
fn get_raft_peers() -> RaftClusterPeers {
    serde_json::from_str(&get_key(String::from("peers"))).unwrap()
}

pub fn setup_tcp_listener(is_leader: bool) {

    IS_LEADER.set(is_leader);

    //available address pool for our Raft Server(s)
    let addrs = [
        SocketAddr::from(([127, 0, 0, 1], 8001)),
        SocketAddr::from(([127, 0, 0, 1], 8002)),
        SocketAddr::from(([127, 0, 0, 1], 8003)),
        SocketAddr::from(([127, 0, 0, 1], 8004)),
        SocketAddr::from(([127, 0, 0, 1], 8005)),
    ];

    let bind_addresses = Addrs{addresses: addrs};

    //bind TcpListener to first available address/port from ADDRS
    let tcp_listener = TcpListener::bind(&bind_addresses.addresses[..]).unwrap();

    //initialize server's message queue
    init_message_queue();

    //initialize raft_log
    initialize_raft_log();

    //print out connection info
    println!("TCP Listener on address: {:#?}, port: {:#?}",
             tcp_listener.local_addr().unwrap().ip(),
             tcp_listener.local_addr().unwrap().port());

    //set cluster peer addresses in kv store
    set_raft_cluster_peers(bind_addresses, tcp_listener.try_clone().unwrap());

    //do leader workload
    if *IS_LEADER.get::<bool>() {
        do_leader_workload();
    } else {
        //start follower election timer
        init_election_timer();
    }

    //using incoming() which calls the accept() fn for each connection
    for socket in tcp_listener.incoming() {

        match socket {

            Ok(socket) => {
                println!("New connection: {}", socket.peer_addr().unwrap());

                //spawn a new thread for each connection
                thread::spawn(move|| {
                    // connection succeeded
                    connection_handler(socket);
                });
            }
            Err(e) => {
                println!("Connection Failed, Error: {}", e);
            }
        }
    }

    // close the socket server using core::mem drop()
    drop(tcp_listener);
}

fn init_election_timer() {
    let handler = thread::spawn(move|| {
        let mut i = 10;
        let timer_end = 0;
        let step = 1;
        println!("Initializing ELECTION_TIMER...");
        while i > timer_end {
            println!("Counting Down Election Timer...");

            i -= step;

            let sleep_time = time::Duration::from_millis(1000);
            time::Instant::now();
            thread::sleep(sleep_time);
        }

        //trigger RequestVote message
        println!("ELECTION_TIMER countdown completed; Server in CANDIDATE_STATE, Triggering Election");
        //TODO send RequestVoteMessage
        broadcast_request_vote();
    });

    //join thread handling election timer
    match handler.join() {
        Ok(_) => {
            //restart the election timer
            init_election_timer();
        }
        Err(e) => {println!("Error Joining Election Timer Thread: {:#?}", e)}
    }
}

fn broadcast_request_vote() {
    //get the raft_log
    let raft_peers = get_raft_peers();
    for peer in raft_peers.addresses {
        //println!("{:#?}", peer.to_string());
        let handler = thread::spawn(move|| {
            let mut raft_log: TheLog = get_raft_log();
            let request_vote_payload = generate_request_vote_payload(raft_log);
            request_vote(peer.to_string(), request_vote_payload);
        });
        handler.join().expect("Failed to join handler thread");
    }

}

//REQUEST_VOTE Request
pub fn request_vote(dest_addr: String, request_vote: RequestVotePayload) {
    match TcpStream::connect(dest_addr) {
        Ok(mut stream) => {
            let msg = RequestVoteMessage {
                src_addr: stream.local_addr().unwrap(),
                src_id: calculate_hash(&stream.local_addr().unwrap()),
                msg_type: MessageType::REQUEST_VOTE,
                payload: request_vote,
            };

            let serialized_bytes = serialize_request_vote(msg);
            stream.write(&*serialized_bytes.as_bytes()).unwrap();

            handle_resp(stream);
        }
        Err(e) => {
            println!("Failed to Connect to Server: {:#?}", e)
        }
    }
}