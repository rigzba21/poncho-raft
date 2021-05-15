use std::net::{TcpStream, Shutdown, TcpListener,  SocketAddr};
use crate::{kv_store};
use crate::{message};
use crate::{log};
use std::io::{Read, Write, Error};
use std::{thread, time};
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use crate::message::MessageType::APPEND_ENTRY;
use crate::message::{MessageType, AppendEntryRequestMessage, Message, init_message_queue, write_new_message_queue, RequestVoteMessage, RequestVoteReplyMessage, RequestVotePayload, RequestVoteReplyPayload, calculate_hash};
use crate::kv_store::{kv_db_setup, set_key, get_key};
use crate::leader::{IS_LEADER};
use crate::log::{initialize_raft_log, get_raft_log};

//main connection handler
pub fn connection_handler(mut socket: TcpStream) {
    let mut data_buffer = [0 as u8; 1024]; //buffer to read in messages

    match socket.read(&mut data_buffer) {
        Ok(size) => {
            let msg = String::from_utf8(Vec::from(&data_buffer[0..size])).expect("Found Invalid UTF-8");

            println!("{:#?}", msg);

            if msg.contains("HEALTHCHECK") {
                let mut socket_clone = socket.try_clone().unwrap();

                healthcheck_handler(socket_clone);
            }
            if msg.contains("APPEND_ENTRY") {
                let message: AppendEntryRequestMessage = serde_json::from_str(&msg).unwrap();
                let mut socket_clone = socket.try_clone().unwrap();

                let mut deque= get_message_queue();

                append_entry_handler(socket_clone, deque, message);
            }
            if msg.contains("REQUEST_VOTE") {
                let message: RequestVoteMessage = serde_json::from_str(&msg).unwrap();
                let mut socket_clone = socket.try_clone().unwrap();

                request_vote_handler(socket_clone, message);
            }
        }
        Err(_) => {
            println!("Terminating Connection: {:#?}", socket.peer_addr().unwrap());
            socket.shutdown(Shutdown::Both).unwrap();
        }
    }
}

//server handler for HEALTHCHECK requests
fn healthcheck_handler(mut socket: TcpStream) {
    let resp_msg = message::get_healthcheck_resp_msg();
    let serialized_resp = serde_json::to_string(&resp_msg).unwrap();
    println!("Response Message: {:#?}", serialized_resp);
    let resp_bytes = serialized_resp.as_bytes();
    socket.write(&resp_bytes[0..resp_bytes.len()]).unwrap();
    socket.flush().unwrap();
}

//server handler for APPEND_ENTRY requests from leader
//here we need to add the message to the individual server's message_queue, and send an arbitrary "ok" HEALTHCHECK response
fn append_entry_handler(mut socket: TcpStream, mut deque: VecDeque<AppendEntryRequestMessage>,
                        message: AppendEntryRequestMessage) {

    //send a generic "ok" response
    healthcheck_handler(socket);

    //add the message to the message_queue
    deque.push_back(message);
    write_new_message_queue(deque);
}

//server handler for REQUEST_VOTE
fn request_vote_handler(mut socket: TcpStream, message: RequestVoteMessage) {
    //follower handles proposed leader's request vote...
    //for now just return a granted: true response to vote reqeust
    let granted = RequestVoteReplyPayload {
        term: message.payload.term,
        granted: true,
    };
    let reply_msg = RequestVoteReplyMessage {
        src_id: calculate_hash(&socket.local_addr().unwrap()),
        src_addr: socket.local_addr().unwrap(),
        msg_type: MessageType::REQUEST_VOTE,
        payload: granted
    };

    let serialized_reply = serde_json::to_string(&reply_msg).unwrap();
    //println!("Response Message: {:#?}", serialized_resp);
    let reply_bytes = serialized_reply.as_bytes();
    socket.write(&reply_bytes[0..reply_bytes.len()]).unwrap();
    socket.flush().unwrap();

}
//serialize generic message
pub fn serialize_msg(message: message::Message) -> String {
    serde_json::to_string(&message).unwrap()
}

//serialize AppendEntry message
pub fn serialize_append_entry(append_entry_message: AppendEntryRequestMessage) -> String {
    serde_json::to_string(&append_entry_message).unwrap()
}

//serialize RequestVote message
pub fn serialize_request_vote(request_vote_message: RequestVoteMessage) -> String {
    serde_json::to_string(&request_vote_message).unwrap()
}

//returns the current deque message queue
pub fn get_message_queue() -> VecDeque<AppendEntryRequestMessage>{
    let deque_string = get_key(String::from("msg_queue"));
    serde_json::from_str(&deque_string).unwrap()
}

//example function for client handling of server response
pub fn handle_resp(mut stream: TcpStream) {
    //handle server response
    let mut resp = [0 as u8; 1024]; // response buffer
    match stream.read(&mut resp) {
        Ok(size) => {
            println!("{:#?}", String::from_utf8(Vec::from(&resp[0..size])).unwrap())
        },
        Err(e) => {
            println!("Failed to receive data: {}", e);
        }
    }
}


#[test]
fn test_serialize_message() {
    let message = message::Message {
        src_id: 1234567890,
        src_addr: SocketAddr::from(([10, 1, 0, 1], 1234)),
        msg_type: message::MessageType::HEALTHCHECK,
        payload: String::from("ok"),
    };

    let copy = message.clone();

    let serialized_messasge = serialize_msg(message);
    let serialized_copy = serialize_msg(copy);
    println!("{:#?}", serialized_messasge);

    let message_hash = message::calculate_hash(&serialized_messasge);
    let copy_hash = message::calculate_hash(&serialized_copy);

    assert_eq!(message_hash, copy_hash);

}