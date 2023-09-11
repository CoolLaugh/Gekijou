use std::io::Cursor;
use regex::Regex;
use reqwest;
use serde::{Deserialize, Serialize};
use xml;
use crate::{anime_data::AnimeInfo, GLOBAL_ANIME_DATA};



#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RssEntry {
    pub title: String,
    pub link: String,
    pub guid: String,
    pub pub_date: String,
    pub downloads: i32,
    pub info_hash: String,
    pub category_id: String,
    pub size: i32,
    pub size_string: String,

    pub derived_values: DerivedValues,
}



#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DerivedValues {
    pub episode: i32,
    pub length: i32,
    pub resolution: i32,
    pub sub_group: String,
    pub anime_id: i32,
    pub title: String,
    pub batch: bool,
}



// search nyaa.si for a anime and returns all entries in a struct format
pub async fn get_rss(anime_id: i32) -> Vec<RssEntry> {

    let media = match GLOBAL_ANIME_DATA.lock().await.get_anime_data(anime_id).await {
        Ok(anime) => anime,
        Err(error) => {
            println!("{}", error);
            return Vec::new();
        }
    };

    let search = if media.title.romaji.is_some() {
        media.title.romaji.unwrap().clone().replace(" ", "+")
    } else {
        return Vec::new();
    }; 
    
    let url = format!("https://nyaa.si/?page=rss&q={}&c=1_2&f=0", search);

    let response = reqwest::get(url).await.unwrap().text().await.unwrap()
        .replace("\n", "")
        .replace("\t", "");

    let cursor = Cursor::new(response);
    
    // Parse the XML document
    let doc = xml::reader::EventReader::new(cursor);

    // Iterate through the events in the XML document
    let mut entry: RssEntry = RssEntry::default(); // temporary holds the entry being parsed
    let mut entrys: Vec<RssEntry> = Vec::new();
    let mut element_name = String::new(); // tracks what property a value belongs to
    for event in doc {
        match event {
            // get the name of the next attribute
            Ok(xml::reader::XmlEvent::StartElement { name, attributes: _, .. }) => {
                element_name = name.local_name;
            }
            // get the value of the next attribute
            Ok(xml::reader::XmlEvent::Characters(text)) => {
                match element_name.as_str() {
                    "title" => { entry.title = text; },
                    "link" => { entry.link = text; },
                    "guid" => { entry.guid = text; },
                    "pubDate" => { entry.pub_date = text; },
                    "downloads" => { entry.downloads = text.parse().unwrap(); },
                    "infoHash" => { entry.info_hash = text; },
                    "categoryId" => { entry.category_id = text; },
                    "size" => { entry.size_string = text; },
                    &_ => (),
                }
            }
            // current entry is completed, store it and reset temp variables
            Ok(xml::reader::XmlEvent::EndElement { name }) => {
                if name.local_name == "item" {
                    entrys.push(entry);
                    entry = RssEntry::default();
                }
                element_name = String::new();
            }
            _ => {}
        }
    }

    let valid_file_extensions = Regex::new(r"[_ ]?(\.mkv|\.avi|\.mp4)").unwrap();
    let file_size = Regex::new(r"(\d{1,3}\.\d?)").unwrap();
    let anime_data = GLOBAL_ANIME_DATA.lock().await;
    for e in entrys.iter_mut() {

        let title = e.title.clone();
        if let Some(identify_info) = anime_data.identify_anime(title, None) {
            println!("{:?}", e);
            println!("{:?}", identify_info);
        }
        e.derived_values.title = e.title.clone();

        // title = valid_file_extensions.replace_all(&title, "").to_string();

        // e.derived_values.resolution = file_name_recognition::extract_resolution(&title);

        // e.derived_values.sub_group = file_name_recognition::extract_sub_group(&title);

        // title = file_name_recognition::remove_brackets(&title);

        // let (episode_string, episode, length) = file_name_recognition::identify_number(&title);
        // e.derived_values.length = length;
        // e.derived_values.episode = episode;
        // title = title.replace(&episode_string, "");

        // let lowercase_title = title.to_ascii_lowercase();
        // let (mut identified_anime_id, mut identified_title, mut similarity) = file_name_recognition::identify_media_id(&lowercase_title, &anime_data, Some(anime_id));
        // //println!("{} {} {} ", identified_anime_id, identified_title, similarity);
        // if identified_anime_id == 0 {
        //     (identified_anime_id, identified_title, similarity) = file_name_recognition::identify_media_id(&lowercase_title, &anime_data, None);
        //     //print!("{} {} {} ", identified_anime_id, identified_title, similarity)
        // }
        // if similarity > 0.0 {
        //     e.derived_values.anime_id = identified_anime_id;
        //     e.derived_values.title = identified_title;
        // }

        // let captures = file_size.captures(&e.size_string).unwrap();
        // let size: f64 = captures.get(1).unwrap().as_str().parse().unwrap();

        // if e.size_string.contains("GiB") {
        //     e.size = (size * 1024.0 * 1024.0) as i32;
        // } else if e.size_string.contains("MiB") {
        //     e.size = (size * 1024.0) as i32;
        // } else {
        //     e.size = size as i32;
        // }

        // e.derived_values.batch = identify_batch(&e.title, e.derived_values.episode, e.size);
    }

    entrys
}



// returns true if a filename is of a batch of episodes
fn identify_batch(filename: &String, episode: i32, size: i32) -> bool {

    let season = Regex::new(r"[Ss]eason ?\d+").unwrap();
    let season_short = Regex::new(r"[Ss] ?\d").unwrap();
    let episode_number = Regex::new(r" - \d+").unwrap();
    let season_short_not = Regex::new(r"[Ss] ?\d+[Ee]\d+").unwrap();
    let episode_range = Regex::new(r"0?1 ?[-~] ?\d+").unwrap();
    let batch = Regex::new(r"[Bb]atch").unwrap();

    if batch.is_match(filename) {
        return true;
    }
    if episode_range.is_match(filename) {
        return true;
    }
    if episode_number.is_match(filename) {
        // must be before season check because some files have both a episode number and a season number
        return false;
    }
    if season.is_match(filename) {
        return true;
    }
    if season_short.is_match(filename) && season_short_not.is_match(filename) == false {
        return true;
    }
    if episode == 0 {
        return true;
    }
    if size > 3 * 1024 * 1024 /* 3GB */ {
        return true;
    }

    false
}