use std::collections::{HashMap, HashSet};
use std::fs::{self, create_dir};
use std::fs::File;
use std::io::{Write, Read};
use std::path::Path;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tauri::async_runtime::Mutex;

use crate::api_calls::{AnimeInfo, UserAnimeInfo};
use crate::{GLOBAL_ANIME_DATA, GLOBAL_USER_ANIME_DATA, GLOBAL_USER_ANIME_LISTS, GLOBAL_USER_SETTINGS, GLOBAL_TOKEN, GLOBAL_ANIME_PATH, GLOBAL_REFRESH_UI, GLOBAL_ANIME_UPDATE_QUEUE};

extern crate dirs;

#[cfg(debug_assertions)]
const GEKIJOU_FOLDER: &str = "Gekijou_debug";

#[cfg(not(debug_assertions))]
const GEKIJOU_FOLDER: &str = "Gekijou";

// writes access token data to a file
pub async fn write_file_token_data() {
    write_file_data_mutex(&GLOBAL_TOKEN, "token").await;
}

// read access token data from the file
pub async fn read_file_token_data() -> Result<(), &'static str> {
    read_file_data(&GLOBAL_TOKEN, "token").await
}

// writes user settings to a file
pub async fn write_file_user_settings() {
    write_file_data_mutex(&GLOBAL_USER_SETTINGS, "user_settings").await;
}

// reads user settings out of a file
pub async fn read_file_user_settings() -> Result<(), &'static str> {
    read_file_data(&GLOBAL_USER_SETTINGS, "user_settings").await
}

// writes all held data on anime to a file
pub fn write_file_anime_info_cache(anime_data: &HashMap<i32, AnimeInfo>) {
    write_file_data(anime_data, "anime_cache");
}

// reads all stored data on anime from a file
pub async fn read_file_anime_info_cache() -> Result<(), &'static str> {
    read_file_data(&GLOBAL_ANIME_DATA, "anime_cache").await
}

pub async fn write_file_user_info() {
    write_file_data_mutex(&GLOBAL_USER_ANIME_DATA, "user_data").await;
    write_file_data_mutex(&GLOBAL_USER_ANIME_LISTS, "user_lists").await;
}

pub async fn read_file_user_info() -> Result<(), &'static str> {
    match read_file_data(&GLOBAL_USER_ANIME_DATA, "user_data").await {
        Ok(_result) => {
            read_file_data(&GLOBAL_USER_ANIME_LISTS, "user_lists").await
        },
        Err(error) => return Err(error),
    }
}

pub async fn write_file_episode_path() {
    write_file_data_mutex(&GLOBAL_ANIME_PATH, "episode_path").await;
}

pub async fn read_file_episode_path() -> Result<(), &'static str> {
    read_file_data(&GLOBAL_ANIME_PATH, "episode_path").await
}

pub async fn write_file_update_queue(update_queue: &Vec<UserAnimeInfo>) {
    write_file_data(&update_queue, "update_queue");
}

pub async fn read_file_update_queue() -> Result<(), &'static str> {
    read_file_data(&GLOBAL_ANIME_UPDATE_QUEUE, "update_queue").await
}

pub async fn write_file_known_files(known_files: &HashSet<String>) {
    write_file_data(&known_files, "known_files");
}

pub async fn read_file_known_files(known_files: &Mutex<HashSet<String>>) -> Result<(), &'static str> {
    read_file_data(known_files, "known_files").await
}

pub async fn write_file_404_ids(not_found_ids: &HashSet<i32>) {
    write_file_data(&not_found_ids, "404_ids");
}

pub async fn read_file_404_ids(not_found_ids: &Mutex<HashSet<i32>>) -> Result<(), &'static str> {
    read_file_data(&not_found_ids, "404_ids").await
}

// writes all held data on anime to a file
fn write_file_data<T: Serialize>(global: &T, filename: &str) {
    
    let file_location = format!("{}/{}/{}.json", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER, filename);
    let file_backup_location = format!("{}/{}/{}_backup.json", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER, filename);
    let file_path = Path::new(&file_location);
    let file_backup_path = Path::new(&file_backup_location);
    let mut file: File;

    // backup file before replacing it
    if file_path.exists() {

        // remove backup so we can make a new one
        if file_backup_path.exists() {
            match fs::remove_file(file_backup_path) {
                Err(why) => panic!("unable to remove, {}", why),
                Ok(file) => file,
            };
        }

        // change file into backup file
        match fs::rename(file_path, file_backup_path) {
            Err(why) => panic!("unable to move, {}", why),
            Ok(file) => file,
        };
    }

    if gekijou_folder_exists_or_created() {

        // create the file
        file = match File::create(file_path) {
            Err(why) => panic!("unable to open, {}", why),
            Ok(file) => file,
        };

        // write contents into file
        match file.write_all(serde_json::to_string(global).unwrap().as_bytes()) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
    }
}



async fn write_file_data_mutex<T: Serialize>(global: &Mutex<T>, filename: &str) {
    write_file_data(&*global.lock().await, filename);
}



// reads all stored data from a file into the global collection
async fn read_file_data<T: DeserializeOwned>(global: &Mutex<T>, filename: &str) -> Result<(), &'static str> {

    let file_location = format!("{}/{}/{}.json", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER, filename);
    let file_path = Path::new(&file_location);
    
    if file_path.exists() {

        // open the file
        let mut file = match File::open(&file_path) {
            Err(_why) => {
                GLOBAL_REFRESH_UI.lock().await.errors.push(String::from(filename.to_owned() + " Can't open file"));
                // try to use the backup file if the file doesn't work
                return read_backup_file_data(global, filename).await;
            },
            Ok(file) => file,
        };
    
        // read all data out of the file
        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Err(_why) => {
                GLOBAL_REFRESH_UI.lock().await.errors.push(String::from(filename.to_owned() + " Can't read file"));
                // try to use the backup file if the file doesn't work
                return read_backup_file_data(global, filename).await;
            },
            Ok(file) => file,
        };
        
        match serde_json::from_str(&buffer) {
            Ok(result) => {
                *global.lock().await = result;
            },
            Err(_error) => {
                GLOBAL_REFRESH_UI.lock().await.errors.push(String::from(filename.to_owned() + " Can't process json"));
                // try to use the backup file if the file doesn't work
                return read_backup_file_data(global, filename).await;
            },
        }
    }

    Ok(())
}



// reads all stored data from a backup file into the global collection
async fn read_backup_file_data<T: DeserializeOwned>(global: &Mutex<T>, filename: &str) -> Result<(), &'static str> {

    let file_backup_location = format!("{}/{}/{}_backup.json", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER, filename);
    let file_backup_path = Path::new(&file_backup_location);

    if file_backup_path.exists() {

        // open the file
        let mut file = match File::open(&file_backup_path) {
            Err(_why) => {
                return Err("Can't open backup file");
            },
            Ok(file) => file,
        };
    
        // read all data out of the file
        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Err(_why) => { return Err("Can't read backup file")},
            Ok(file) => file,
        };
        
        match serde_json::from_str(&buffer) {
            Ok(result) => {
                *global.lock().await = result;
            },
            Err(_error) => { return Err("Can't process backup json")},
        }
    }

    Ok(())
}



// checks if gekijou folder exists, if it does not exist it will try to create it
// returns true if the folder exists
fn gekijou_folder_exists_or_created() -> bool {

    let folder = dirs::config_dir().unwrap().join(GEKIJOU_FOLDER);
    if folder.exists() {
        true
    } else {
        match create_dir(folder) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(_e) => true,
        }
    }
}



// delete all files that store information between sessions
pub fn delete_data() -> bool {

    let files = vec!["token","token_backup","user_settings","user_settings_backup","anime_cache","anime_cache_backup",
                                "user_data","user_data_backup","user_lists","user_lists_backup","episode_path","episode_path_backup",
                                "update_queue","update_queue_backup","known_files","known_files_backup","404_ids","404_ids_backup"];

    for file in files {

        let file_location = format!("{}/{}/{}.json", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER, file);
        let file_path = Path::new(&file_location);
        if file_path.exists() {
            match fs::remove_file(file_path) {
                Err(_why) => return false,
                Ok(file) => file,
            };
        }
    }

    true
}