use std::{path::Path, fs::File, io::Read};
use serde::{Serialize, Deserialize};

use crate::{GLOBAL_ANIME_DATA, anime_data::IdentifyInfo};



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
    
    let mut anime_data = GLOBAL_ANIME_DATA.lock().await;
    let filenames_values: Vec<serde_json::Value> = serde_json::from_str(&buffer).unwrap();
    let mut test_results: Vec<FilenameTest> = Vec::new();
    let mut anime_ids: Vec<i32> = Vec::new();

    filenames_values.iter().for_each(|entry| {

        let test_result = FilenameTest { 
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
        };

        anime_ids.push(test_result.expected_anime_id);
        test_results.push(test_result);
    });

    anime_data.get_anime_list_data(anime_ids).await;

    test_results.iter_mut().for_each(|entry| {

        let identify_info: Option<IdentifyInfo> = anime_data.identify_anime(entry.filename.clone(), None);

        if let Some(info) = identify_info {
            entry.anime_id = info.media_id;
            entry.similarity_score = info.similarity_score;
            entry.title = info.file_title;
            entry.episode = info.episode;
            entry.length = info.episode_length;
            entry.resolution = info.resolution;
            entry.id_title = info.media_title;
        }
    });

    test_results
}