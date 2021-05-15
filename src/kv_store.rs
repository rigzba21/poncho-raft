use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use std::error::Error;
use std::path::Path;

pub fn kv_db_setup() -> Result<PickleDb, Box<dyn Error>> {
    match check_db_exists() {
        Ok(true) => {
            let db = PickleDb::load("kv.db", PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json).unwrap();
            Ok(db)
        }
        Ok(false) => {
            let db = PickleDb::new("kv.db", PickleDbDumpPolicy::AutoDump, SerializationMethod::Json);
            Ok(db)
        }
        _ => {panic!()}
    }
}

fn check_db_exists() -> Result<bool, Box<dyn Error>> {
    Ok(Path::new("kv.db").exists())
}

pub fn get_key(key: String) -> String {
    let db = kv_db_setup().unwrap();
    let value = db.get::<String>(&*key);
    if value == None {
        String::from("Bad-Key!")
    } else {
        value.unwrap()
    }
}

pub fn set_key(key: String, value: String) {
    let mut db = kv_db_setup().unwrap();
    db.set(&*key, &value).unwrap()
}

pub fn delete_key(key: String) -> pickledb::error::Result<bool> {
    let mut db = kv_db_setup().unwrap();
    db.rem(&*key)
}