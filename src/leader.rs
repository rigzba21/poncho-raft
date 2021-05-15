//module for Raft Leader Request functionality

use std::net::{TcpStream, SocketAddr};
use crate::message;
use crate::log;
use crate::connection_handler;
use std::io::Write;
use serde::{Serialize, Deserialize};
use crate::message::{RequestVoteMessage, calculate_hash, MessageType, Message, AppendEntryRequestMessage, RequestVotePayload, write_new_message_queue, process_next_message};
use crate::connection_handler::{serialize_msg, handle_resp, serialize_append_entry,
                                serialize_request_vote, get_message_queue};
use std::collections::VecDeque;
use crate::log::{LogEntry, TheLog, validate_log_entry};
use crate::kv_store::get_key;
use std::{thread, time};

pub static IS_LEADER: state::Container = state::Container::new();


//generic healthcheck for testing
//use this for the HEARTBEAT
pub fn send_healthcheck_message(dest_addr: String) {
    match TcpStream::connect(dest_addr) {
        Ok(mut stream) => {
            let msg = Message {
                src_addr: stream.local_addr().unwrap(),
                src_id: calculate_hash(&stream.local_addr().unwrap()),
                msg_type: MessageType::HEALTHCHECK,
                payload: String::from("ok"),
            };

            let serialized_bytes = serialize_msg(msg);
            stream.write(&*serialized_bytes.as_bytes()).unwrap(); //stream key to tcp server

            //this is a function for the calling "client" to handle the server response message
            handle_resp(stream);
        }
        Err(e) => {
            println!("Failed to Connect to Server: {:#?}", e)
        }
    }
}

//APPEND_ENTRY Request
pub fn append_entry_request(dest_addr: String, append_entry_req: message::AppendEntryRequest) {
    match TcpStream::connect(dest_addr) {
        Ok(mut stream) => {
            let msg = AppendEntryRequestMessage {
                src_addr: stream.local_addr().unwrap(),
                src_id: calculate_hash(&stream.local_addr().unwrap()),
                msg_type: MessageType::APPEND_ENTRY,
                payload: append_entry_req,
            };

            let serialized_bytes = serialize_append_entry(msg);
            stream.write(&*serialized_bytes.as_bytes()).unwrap();

            handle_resp(stream);
        }
        Err(e) => {
            println!("Failed to Connect to Server: {:#?}", e)
        }
    }
}

pub fn do_leader_workload() {
    thread::spawn(move|| {
        println!("Separate Thread for Leader Stuff");
        while *IS_LEADER.get::<bool>() {
            println!("doing leader stuff");


            process_next_message();
            let sleep_time = time::Duration::from_millis(10000);
            time::Instant::now();
            thread::sleep(sleep_time);
        }
    });
}



