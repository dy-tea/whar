use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Header {
    pub version: String,
    pub file_names: Vec<String>,
    pub file_sizes: Vec<usize>,
    pub hash_map: HashMap<char, String>,
}

impl Header {
    pub fn new(
        version: String,
        file_names: Vec<String>,
        file_sizes: Vec<usize>,
        hash_map: HashMap<char, String>,
    ) -> Self {
        Header {
            version,
            file_names,
            file_sizes,
            hash_map,
        }
    }

    pub fn get_serialized(&self) -> Vec<u8> {
        match serialize(&self) {
            Ok(s) => s,
            Err(e) => {
                println!("ERROR: Could not serialize header, {}", e);
                panic!("FATAL ERROR");
            }
        }
    }

    pub fn get_deserialized(input: Vec<u8>) -> Header {
        match deserialize(&input) {
            Ok(d) => d,
            Err(e) => {
                println!("ERROR: Could not deserialize header, {}", e);
                panic!("FATAL ERROR");
            }
        }
    }
}
