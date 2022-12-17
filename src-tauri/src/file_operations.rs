use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Write, Read};
use std::path::Path;

use crate::api_calls::TokenData;

pub fn write_file_token_data(token_data: &TokenData) {

    let path = Path::new("token.txt");
    let mut file: File;

    if path.exists() == false {
        file = match File::create(path) {
            Err(why) => panic!("unable to open {}", why),
            Ok(file) => file,
        };
    } else {
        file = match OpenOptions::new().write(true).open("token.txt") {
            Err(why) => panic!("unable to open {}", why),
            Ok(file) => file,
        };
    }

    match file.write_all(serde_json::to_string(token_data).unwrap().as_bytes()) {
        Err(why) => panic!("ERROR: {}", why),
        Ok(file) => file,
    };
}

pub fn read_file_token_data() -> TokenData {

    let path = Path::new("token.txt");
    let mut file = match File::open(&path) {
        Err(why) => panic!("unable to open {}", why),
        Ok(file) => file,
    };
    print!("Reading file");

    let mut buffer = String::new();
    match file.read_to_string(&mut buffer) {
        Err(why) => panic!("ERROR: {}", why),
        Ok(file) => file,
    };

    let token_data: TokenData = serde_json::from_str(&buffer).unwrap();
    token_data
}

pub fn token_data_file_exists() -> bool {

    let path = Path::new("token.txt");
    path.exists()
}