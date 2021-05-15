// message queue for leaders to accept messages
use std::collections::VecDeque;
use serde::{Serialize, Deserialize};
use std::net::{SocketAddr, Ipv4Addr};
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::borrow::Borrow;
use crate::log;
use crate::kv_store::{kv_db_setup, set_key, get_key};
use crate::log::{TheLog, validate_log_entry, LogEntry};
use crate::connection_handler::get_message_queue;

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub enum MessageType {
    HEALTHCHECK,
    APPEND_ENTRY,
    REQUEST_VOTE,
}

//generic message
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Message {
    pub src_id: u64,
    pub src_addr: SocketAddr,
    pub msg_type: MessageType,
    pub payload: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct HealthcheckResponse {
    pub msg_type: MessageType,
    pub payload: String,
}

//struct for the APPEND_ENTRY request
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct AppendEntryRequest {
    pub log_entry: log::LogEntry,
    pub entries: log::TheLog,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct AppendEntryRequestMessage {
    pub src_id: u64,
    pub src_addr: SocketAddr,
    pub msg_type: MessageType,
    pub payload: AppendEntryRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct RequestVoteMessage {
    pub src_id: u64,
    pub src_addr: SocketAddr,
    pub msg_type: MessageType,
    pub payload: RequestVotePayload,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct RequestVotePayload {
    pub last_log_index: i32,
    pub last_log_term: i32,
    pub term: i32
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct RequestVoteReplyPayload {
    pub term: i32,
    pub granted: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct RequestVoteReplyMessage {
    pub src_id: u64,
    pub src_addr: SocketAddr,
    pub msg_type: MessageType,
    pub payload: RequestVoteReplyPayload,
}

//initialize the server's message queue
pub fn init_message_queue() {
    let msg_queue: VecDeque<AppendEntryRequestMessage> = Default::default();
    let serialized = serde_json::to_string(&msg_queue).unwrap();
    set_key(String::from("msg_queue"), serialized);
}

//process next APPEND_ENTRY request message
pub fn process_next_message() {
    //pop_front from message_queue
    let mut deque = get_message_queue();
    if deque.is_empty() {
        println!("Message Queue is Empty...Continuing");
        return
    }
    let next_message = deque.pop_front().unwrap();
    println!("Message Popped off Queue, new Size: {}", deque.len());
    let updated_msg_queue = serde_json::to_string(&deque).unwrap();
    set_key("msg_queue".parse().unwrap(), updated_msg_queue);

    let log_entry: LogEntry = next_message.payload.log_entry;

    //process log entry before appending to log
    let mut raft_log: TheLog = serde_json::from_str(&get_key(String::from("raft_log"))).unwrap();

    //if log is empty, add message
    if raft_log.log_entries.clone().is_empty() {
        raft_log.log_entries.push_back(log_entry.clone());
        let serialized = serde_json::to_string(&raft_log).unwrap();
        set_key(String::from("raft_log"), serialized);
        println!("Log Appended: {:#?}", log_entry.clone());
        return
    }

    let is_valid_log_entry = validate_log_entry(log_entry.clone(), raft_log.clone());

    if is_valid_log_entry {
        println!("Log Entry is valid: {:#?}", log_entry.clone());
    }
    else {
        println!("Invalid Log Entry: {:#?}", log_entry.clone());
    }
}

pub fn write_new_message_queue(deque: VecDeque<AppendEntryRequestMessage>) {
    let serialized = serde_json::to_string(&deque).unwrap();
    set_key(String::from("msg_queue"), serialized);
}

//generate a RequestVotePayload
pub fn generate_request_vote_payload(mut raft_log: TheLog) -> RequestVotePayload {
    if raft_log.log_entries.is_empty() {
        return RequestVotePayload{
            last_log_index: 0,
            last_log_term: 0,
            term: 1,
        }
    } else {
        let last_log = raft_log.log_entries.pop_back().unwrap();
        RequestVotePayload{
            last_log_index: last_log.leader_commit_index,
            last_log_term: last_log.leader_term,
            term: last_log.leader_term + 1, //increment term
        }
    }
}


pub fn calculate_hash<T: Hash> (t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

pub fn get_healthcheck_resp_msg() -> HealthcheckResponse {
    HealthcheckResponse {
        msg_type: MessageType::HEALTHCHECK,
        payload: String::from("ok"),
    }
}

pub fn get_dummy_append_entry_req() -> AppendEntryRequest {
    let dummy_log_entry = log::LogEntry {
        leader_term: 1,
        leader_id: String::from("1234"),
        prev_index: 0,
        prev_term: 0,
        leader_commit_index: 1,
    };

    let mut log : VecDeque<log::LogEntry> = VecDeque::new();
    log.push_back(dummy_log_entry.clone());

    AppendEntryRequest{
        log_entry: dummy_log_entry,
        entries: log::TheLog{log_entries: log}
    }
}

pub fn get_dummy_request_vote() -> RequestVotePayload {
    RequestVotePayload {
        last_log_index: 1,
        last_log_term: 1,
        term: 1,
    }
}



#[test]
fn test_push_new_message() {
    let message = Message {
        src_id: 1234567890,
        src_addr: SocketAddr::from(([10, 1, 0, 1], 1234)),
        msg_type: MessageType::HEALTHCHECK,
        payload: String::from("ok"),
    };

    let mut deque:VecDeque<Message> = VecDeque::new();

    deque.push_back(message);

    assert_eq!(deque.len(), 1);

}


#[test]
fn test_pop_message() {
    let message = Message {
        src_id: 1234567890,
        src_addr: SocketAddr::from(([10, 1, 0, 1], 1234)),
        msg_type: MessageType::HEALTHCHECK,
        payload: String::from("ok"),
    };

    let mut deque:VecDeque<Message> = VecDeque::new();

    deque.push_back(message.clone());

    let msg_hash = calculate_hash(&message.clone());
    let new_msg_hash = calculate_hash(&deque.pop_front().unwrap());

    assert_eq!(msg_hash, new_msg_hash);

}

