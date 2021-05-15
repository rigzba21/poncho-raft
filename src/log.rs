use serde::{Serialize, Deserialize};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use crate::message::AppendEntryRequest;
use crate::kv_store::{set_key, get_key};

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct LogEntry {
    pub leader_term: i32,
    pub leader_id: String,
    pub prev_index: i32,
    pub prev_term: i32,
    pub leader_commit_index: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct TheLog {
    pub log_entries: VecDeque<LogEntry>,
}

//initialize the serialized raft_log in the kv store
pub fn initialize_raft_log() {
    let mut raft_log = TheLog {
        log_entries: Default::default()
    };
    let serialized: String = serde_json::to_string(&raft_log).unwrap();
    set_key(String::from("raft_log"), serialized);
}

//getter for raft_log
pub fn get_raft_log() -> TheLog {
    serde_json::from_str(&get_key(String::from("raft_log"))).unwrap()
}

pub fn validate_log_entry(entry: LogEntry, mut raft_log: TheLog) -> bool {
    if !check_no_holes_allowed(entry.clone(), raft_log.log_entries.clone()) {
        println!("Invalid prev_index: {}", entry.clone().prev_index);
        return false;
    }
    if !check_prev_term(entry.clone(), raft_log.log_entries.clone()) {
        println!("Invalid prev_term: {:#?}", entry.clone().prev_term);
        return false;
    }
    if is_duplicate_entry(entry.clone(), raft_log.log_entries.clone()) {
        println!("Duplicate Log Entry: {:#?}", entry.clone());
       return false;
    }
    true
}

fn append_entry(entry: LogEntry, log_entries: VecDeque<LogEntry>) -> Result<TheLog, bool> {

    let entry_clone = entry.clone();
    let mut log_entries_clone = log_entries.clone();

    log_entries_clone.push_back(entry_clone);

    //return a new Log with cloned entries
    Ok(TheLog{log_entries: log_entries_clone})
}

/*
    1) the log is never allowed to have holes in it.
 */
fn check_no_holes_allowed(entry: LogEntry, log: VecDeque<LogEntry>) -> bool {
    let current_index = log.len() as i32 - 1;
    let entry_index = entry.prev_index + 1;
    if entry_index > current_index {
        return false;
    }
    true
}

/*
     2) log-continuity; every append operation must verify that the term number of any previous entry matches an expected
     value. For example, if appending at prev_index 8, the prev_term value must match the value of log[prev_index].term
 */
fn check_prev_term(entry: LogEntry, log: VecDeque<LogEntry>) -> bool {
    let prev_term_entry = entry.prev_term;
    let prev_term_log = log.clone().pop_back().unwrap().prev_term;
    if prev_term_entry == prev_term_log {
        true
    } else {
        false
    }
}

/*
    3) Appending log entries at index 0 always works. This is the start of the log.
    5) Calling append_entry() with an empty list of entries is allowed
 */
fn check_empty_log(log_entries: VecDeque<LogEntry>) -> bool {
    log_entries.is_empty() || log_entries.len() == 0
}


/*
    4) Log appends are idempotent; using hash comparisons
 */
fn is_duplicate_entry(entry: LogEntry, log: VecDeque<LogEntry>) -> bool {
    let mut duplicate_entry: bool = false;
    for log_entry in log.iter() {
        let new_entry_hash = calculate_hash(&entry);
        let log_entry_hash = calculate_hash(&log_entry);
        if new_entry_hash == log_entry_hash {
            duplicate_entry = true;
        }
    }
    duplicate_entry
}

/*
    6) If there are already existing entries, but those entries are from an earlier term,
    the existing entries and everything that follows are deleted, then the new entries are added in their
    place.
 */
fn is_entry_earlier_term(entry: LogEntry, log: VecDeque<LogEntry>) -> bool {
    let mut log_clone = log.clone();
    if entry.leader_term < log_clone.pop_back().unwrap().leader_term {
        return true;
    }
    return false;
}
fn replace_existing_entries_earlier_term(entry: LogEntry) -> VecDeque<LogEntry> {
    let mut new_log: VecDeque<LogEntry> = VecDeque::new();
    new_log.push_back(entry);
    new_log
}


fn calculate_hash<T: Hash> (t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_log() {
        let empty_log: VecDeque<LogEntry> = VecDeque::new();
        assert_eq!(check_empty_log(empty_log), true);
    }

    #[test]
    fn test_empty_log_append() {
        let empty_log: VecDeque<LogEntry> = VecDeque::new();

        if check_empty_log(empty_log.clone()) {
            let _log_entry = LogEntry{
                leader_term: 0,
                leader_id: String::from("test"),
                prev_index: 0,
                prev_term: 0,
                leader_commit_index: 0,
            };

            let updated_log = append_entry(_log_entry, empty_log).unwrap();
            println!("{:#?}", updated_log); //print out the log
            assert_eq!(updated_log.log_entries.len(), 1);
        }
    }

    #[test]
    fn test_no_holes() {
        let mut log: VecDeque<LogEntry> = VecDeque::new();

        let log_entry = LogEntry{
            leader_term: 0,
            leader_id: String::from("test"),
            prev_index: 0,
            prev_term: 0,
            leader_commit_index: 0,
        };
        log.push_back(log_entry);

        let bad_entry = LogEntry{
            leader_term: 0,
            leader_id: String::from("1234"),
            prev_index: 5,
            prev_term: 0,
            leader_commit_index: 5,
        };

        assert_eq!(check_no_holes_allowed(bad_entry, log), false)
    }

    #[test]
    fn test_check_entry_hash() {
        let log_entry = LogEntry{
            leader_term: 0,
            leader_id: String::from("test"),
            prev_index: 0,
            prev_term: 0,
            leader_commit_index: 0,
        };
        let duplicate_log_entry = LogEntry{
            leader_term: 0,
            leader_id: String::from("test"),
            prev_index: 0,
            prev_term: 0,
            leader_commit_index: 0,
        };

        let log_entry_hash = calculate_hash(&log_entry);
        let duplicate_entry_hash = calculate_hash(&duplicate_log_entry);
        println!("{}", log_entry_hash);
        println!("{}", duplicate_entry_hash);

        let mut log: VecDeque<LogEntry> = VecDeque::new();
        log.push_back(log_entry);

        assert_eq!(log_entry_hash, duplicate_entry_hash);
        assert_eq!(is_duplicate_entry(duplicate_log_entry, log), true)
    }

    #[test]
    fn test_check_previous_term() {
        let log_entry = LogEntry{
            leader_term: 1,
            leader_id: String::from("test"),
            prev_index: 0,
            prev_term: 0,
            leader_commit_index: 1,
        };
        let mut log: VecDeque<LogEntry> = VecDeque::new();

        log.push_back(log_entry);

        let new_entry = LogEntry{
            leader_term: 1,
            leader_id: String::from("test"),
            prev_index: 1,
            prev_term: 0,
            leader_commit_index: 1,
        };

        assert_eq!(check_prev_term(new_entry, log), true)
    }

    #[test]
    fn test_check_entries_earlier_term() {
        let log_entry_1 = LogEntry{
            leader_term: 5,
            leader_id: String::from("test"),
            prev_index: 5,
            prev_term: 5,
            leader_commit_index: 5,
        };

        let log_entry_0 = LogEntry{
            leader_term: 2,
            leader_id: String::from("test"),
            prev_index: 1,
            prev_term: 1,
            leader_commit_index: 2,
        };

        let mut log: VecDeque<LogEntry> = VecDeque::new();
        log.push_back(log_entry_0);
        log.push_back(log_entry_1);

        let new_log_entry = LogEntry{
            leader_term: 3,
            leader_id: String::from("adifferentid"),
            prev_index: 3,
            prev_term: 3,
            leader_commit_index: 3,
        };

        assert_eq!(is_entry_earlier_term(new_log_entry, log), true);
    }

    #[test]
    fn test_replace_entries_earlier_term() {
        let log_entry_1 = LogEntry{
            leader_term: 5,
            leader_id: String::from("test"),
            prev_index: 5,
            prev_term: 5,
            leader_commit_index: 5,
        };

        let log_entry_0 = LogEntry{
            leader_term: 2,
            leader_id: String::from("test"),
            prev_index: 1,
            prev_term: 1,
            leader_commit_index: 2,
        };

        let mut log: VecDeque<LogEntry> = VecDeque::new();
        log.push_back(log_entry_0);
        log.push_back(log_entry_1);

        let new_log_entry = LogEntry{
            leader_term: 3,
            leader_id: String::from("adifferentid"),
            prev_index: 3,
            prev_term: 3,
            leader_commit_index: 3,
        };

       let replaced_log: VecDeque<LogEntry> = replace_existing_entries_earlier_term(new_log_entry);
        assert_eq!(replaced_log.is_empty(), false)
    }

}