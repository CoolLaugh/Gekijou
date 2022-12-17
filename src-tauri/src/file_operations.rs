use std::fs;
use std::fs::File;
use std::io::{Write, Read};
use std::path::Path;

use crate::GLOBAL_ANIME_DATA;
use crate::api_calls::TokenData;
use crate::api_calls::UserSettings;



pub fn write_file_token_data(token_data: &TokenData) {

    let path = Path::new("token.txt");
    let path_backup = Path::new("token_backup.txt");
    let mut file: File;

    // backup file before replacing it
    if path.exists() {

        // remove backup so we can make a new one
        if path_backup.exists() {
            match fs::remove_file(path_backup) {
                Err(why) => panic!("unable to remove, {}", why),
                Ok(file) => file,
            };
        }

        // change file into backup file
        match fs::rename(path, path_backup) {
            Err(why) => panic!("unable to move, {}", why),
            Ok(file) => file,
        };
    }

    // create the file
    file = match File::create(path) {
        Err(why) => panic!("unable to open, {}", why),
        Ok(file) => file,
    };

    // write token data into file
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

    let mut buffer = String::new();
    match file.read_to_string(&mut buffer) {
        Err(why) => panic!("ERROR: {}", why),
        Ok(file) => file,
    };

    let token_data: TokenData =  serde_json::from_str(&buffer).unwrap();
    token_data
}



pub fn token_data_file_exists() -> bool {

    let path = Path::new("token.txt");
    path.exists()
}



pub fn write_file_user_settings(settings: &UserSettings) {

    let path = Path::new("user_settings.txt");
    let path_backup = Path::new("user_settings_backup.txt");
    let mut file: File;

    // backup file before replacing it
    if path.exists() {

        // remove backup so we can make a new one
        if path_backup.exists() {
            match fs::remove_file(path_backup) {
                Err(why) => panic!("unable to remove, {}", why),
                Ok(file) => file,
            };
        }

        // change file into backup file
        match fs::rename(path, path_backup) {
            Err(why) => panic!("unable to move, {}", why),
            Ok(file) => file,
        };
    }

    // create the file
    file = match File::create(path) {
        Err(why) => panic!("unable to open, {}", why),
        Ok(file) => file,
    };

    // write user settings into file
    match file.write_all(serde_json::to_string(settings).unwrap().as_bytes()) {
        Err(why) => panic!("ERROR: {}", why),
        Ok(file) => file,
    };
}



pub fn read_file_user_settings() -> UserSettings {

    let path = Path::new("user_settings.txt");

    if path.exists() {

        let mut file = match File::open(&path) {
            Err(why) => panic!("unable to open {}", why),
            Ok(file) => file,
        };
    
        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
    
        let token_data: UserSettings = serde_json::from_str(&buffer).unwrap();
        token_data
    }
    else {
        return UserSettings::new();
    }
}



pub async fn write_file_anime_info_cache() {

    let path = Path::new("anime_cache.txt");
    let path_backup = Path::new("anime_cache_backup.txt");
    let mut file: File;

    // backup file before replacing it
    if path.exists() {

        // remove backup so we can make a new one
        if path_backup.exists() {
            match fs::remove_file(path_backup) {
                Err(why) => panic!("unable to remove, {}", why),
                Ok(file) => file,
            };
        }

        // change file into backup file
        match fs::rename(path, path_backup) {
            Err(why) => panic!("unable to move, {}", why),
            Ok(file) => file,
        };
    }

    // create the file
    file = match File::create(path) {
        Err(why) => panic!("unable to open, {}", why),
        Ok(file) => file,
    };

    // write user settings into file
    match file.write_all(serde_json::to_string(&*GLOBAL_ANIME_DATA.lock().await).unwrap().as_bytes()) {
        Err(why) => panic!("ERROR: {}", why),
        Ok(file) => file,
    };
}



pub async fn read_file_anime_info_cache() {

    let path = Path::new("anime_cache.txt");

    if path.exists() {

        let mut file = match File::open(&path) {
            Err(why) => panic!("unable to open {}", why),
            Ok(file) => file,
        };
    
        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
    
        *GLOBAL_ANIME_DATA.lock().await = serde_json::from_str(&buffer).unwrap();
    }
}