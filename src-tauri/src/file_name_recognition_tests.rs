use std::{path::Path, fs::File, io::Read};
use serde::{Serialize, Deserialize};
use regex::Regex;

use crate::{file_name_recognition, GLOBAL_ANIME_DATA, api_calls};



#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FilenameTest {

    pub filename: String,

    pub anime_id: i32,
    pub similarity_score: f64,
    pub title: String,
    pub episode: i32,
    pub length: i32,
    pub resolution: i32,

    pub expected_anime_id: i32,
    pub id_title: String,
    pub expected_episode: i32,
    pub expected_resolution: i32,
}



pub async fn filename_tests() -> Vec<FilenameTest> {

    let file_path = Path::new("filename_tests.json");

    if file_path.exists() == false {
        return Vec::new();
    }

    // open the file
    let mut file = match File::open(&file_path) {
        Err(why) => panic!("ERROR: {}", why),
        Ok(file) => file,
    };

    // read all data out of the file
    let mut buffer = String::new();
    match file.read_to_string(&mut buffer) {
        Err(why) => panic!("ERROR: {}", why),
        Ok(file) => file,
    };
    
    let filenames_values: Vec<serde_json::Value> = serde_json::from_str(&buffer).unwrap();
    let mut filenames: Vec<FilenameTest> = Vec::new();
    filenames_values.iter().for_each(|entry| {
        filenames.push(FilenameTest { 
            filename: entry["filename"].as_str().unwrap().to_string(), 
            anime_id: 0, 
            similarity_score: 0.0, 
            title: String::new(), 
            episode: 0, 
            length: 0,
            resolution: 0, 
            expected_anime_id: entry["expected_anime_id"].as_i64().unwrap() as i32, 
            id_title: String::new(), 
            expected_episode: entry["expected_episode"].as_i64().unwrap() as i32, 
            expected_resolution: entry["expected_resolution"].as_i64().unwrap() as i32 
        });
    });

    {
        let mut anime_data = GLOBAL_ANIME_DATA.lock().await;
        let mut missing_ids: Vec<i32> = Vec::new();
        filenames.iter().for_each(|entry| {
            if anime_data.contains_key(&entry.expected_anime_id) == false {
                missing_ids.push(entry.expected_anime_id);
            }
        });
        api_calls::anilist_api_call_multiple(missing_ids, &mut anime_data).await;
    }

    file_name_recognition::get_prequel_data().await;

    let valid_file_extensions = Regex::new(r"[_ ]?(\.mkv|\.avi|\.mp4)").unwrap();
    filenames.iter_mut().for_each(|entry| {
        entry.title = valid_file_extensions.replace_all(&entry.filename, "").to_string();
    });

    filenames.iter_mut().for_each(|entry| {
        entry.resolution = file_name_recognition::extract_resolution(&entry.title);
    });

    filenames.iter_mut().for_each(|entry| {
        entry.title = file_name_recognition::remove_brackets(&entry.title);
    });

    filenames.iter_mut().for_each(|entry| {

        let episode = file_name_recognition::identify_number(&entry.title);
        if episode.1 != 0 {
            entry.episode = episode.1;
            entry.length = episode.2;
            entry.title = entry.title.replace(episode.0.as_str(), "");
        }
    });

    filenames.iter_mut().for_each(|entry| {

        entry.title = file_name_recognition::irrelevant_information_removal(entry.title.clone());
    });

    let anime_data = GLOBAL_ANIME_DATA.lock().await;
    filenames.iter_mut().for_each(|entry| {

        let (id, _title, similarity_score) = file_name_recognition::identify_media_id(&entry.title, &anime_data, None);
        
        if similarity_score > entry.similarity_score {
            entry.anime_id = id;
            entry.similarity_score = similarity_score;
        }
    });

    filenames.iter_mut().for_each(|entry| {

        (entry.anime_id, entry.episode) = file_name_recognition::replace_with_sequel(entry.anime_id, entry.episode, &anime_data);
    });

    filenames.iter_mut().for_each(|entry| {

        file_name_recognition::episode_fix(entry.anime_id, &mut entry.episode, &anime_data);
    });

    filenames.iter_mut().for_each(|entry| {

        entry.id_title = anime_data.get(&entry.anime_id).unwrap().title.romaji.clone().unwrap();
    });

    filenames
}