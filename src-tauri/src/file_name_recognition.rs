use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use regex::Regex;
use serde::{Serialize, Deserialize};


use crate::api_calls::AnimeInfo;
use crate::{GLOBAL_ANIME_DATA, GLOBAL_ANIME_PATH, GLOBAL_USER_SETTINGS, file_operations};
use strsim;

// working struct to store data while determining what anime a file belongs to
#[derive(Debug, Clone, Default)]
struct AnimePathWorking {
    path: String,
    filename: String,
    episode: i32,
    media_id: i32,
    similarity_score: f64,
}

impl AnimePathWorking {
    pub const fn new(new_path: String, new_filename: String) -> AnimePathWorking {
        AnimePathWorking { path: new_path, filename: new_filename, episode: 0, media_id: 0, similarity_score: 0.0 }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AnimePath {
    pub path: String,
    pub similarity_score: f64,
}

// scans these folders and subfolders looking for files that match titles in the users anime list
// found files are then stored in a global list for each anime and episode
// media_id is for finding files for a specific anime instead of any anime known
pub async fn parse_file_names(media_id: Option<i32>) -> bool {
    
    let mut episode_found = false;
    let folders = GLOBAL_USER_SETTINGS.lock().await.folders.clone();
    for folder in folders {

        let path = Path::new(&folder);
        if path.exists() == false {
            continue;
        }

        let mut sub_folders: Vec<String> = Vec::new();
        sub_folders.push(folder.clone());
        let mut file_names: Vec<AnimePathWorking> = Vec::new();

        while sub_folders.is_empty() == false {
            
            let sub_folder = sub_folders.remove(0);
            
            for file in fs::read_dir(sub_folder).unwrap() {

                let unwrap = file.unwrap();
                if unwrap.file_type().unwrap().is_dir() {

                    sub_folders.push(unwrap.path().to_str().unwrap().to_string());
                } else {

                    file_names.push(AnimePathWorking::new(unwrap.path().to_str().unwrap().to_string(), unwrap.file_name().to_str().unwrap().to_string()));
                }
            }
        }
        
        remove_invalid_files(&mut file_names);
        
        // remove brackets and their contents, the name and episode are unlikely to be here
        file_names.iter_mut().for_each(|name| {
            name.filename = remove_brackets(&name.filename);
        });
        
        identify_episode_number(&mut file_names);
        
        irrelevant_information_removal_paths(&mut file_names);

        string_similarity(&mut file_names, media_id).await;
        
        
        let mut file_paths = GLOBAL_ANIME_PATH.lock().await;
        let anime_data = GLOBAL_ANIME_DATA.lock().await;
        for file in file_names {

            if file.similarity_score > 0.8 && file.episode <= anime_data.get(&file.media_id).unwrap().episodes.unwrap() {

                let media = file_paths.entry(file.media_id).or_default();
                if media.contains_key(&file.episode) && media.get(&file.episode).unwrap().similarity_score < file.similarity_score {
                    media.entry(file.media_id).and_modify(|anime_path| {

                        anime_path.similarity_score = file.similarity_score;
                        anime_path.path = file.path;
                    });
                    episode_found = true;
                } else {

                    media.insert(file.episode, AnimePath { path: file.path, similarity_score: file.similarity_score });
                    episode_found = true;
                }
            }
        }
    }

    remove_missing_files().await;

    file_operations::write_file_episode_path().await;
    episode_found
}

// remove all files that no longer exist
async fn remove_missing_files() {

    let mut file_paths = GLOBAL_ANIME_PATH.lock().await;
    for (_, anime) in file_paths.iter_mut() {

        anime.retain(|_, episode| { Path::new(&episode.path).exists() });
    }
    file_paths.retain(|_,anime| { anime.len() > 0 });
}

// remove all brackets from a filename
pub fn remove_brackets(filename: &String) -> String {
    Regex::new(r"((\[[^\[\]]+\]|\([^\(\)]+\))[ _]*)+").unwrap().replace_all(&filename, "").to_string()
}

// removes any files that are the wrong file type or extra (openings, endings, etc)
fn remove_invalid_files(paths: &mut Vec<AnimePathWorking>) {

    // remove files that are not video files
    let valid_file_extensions = Regex::new(r"[_ ]?(\.mkv|\.avi|\.mp4)").unwrap();
    // remove openings, endings, PV, and other non episode videos
    // spell-checker:disable
    let extra_videos = Regex::new(r"[ _\.][oO][pP]\d*([vV]\d)?[ _\.]|[ _\.]NCOP\d*([vV]\d)?[ _\.]|[ _\.]NCED\d*([vV]\d)?[ _\.]|[ _\.][eE][dD]\d*([vV]\d)?[ _\.]|[ _\.][sS]kit[ _\.]|[eE]nding|[oO]pening|[ _][pP][vV][ _]|[bB][dD] [mM][eE][nN][uU]").unwrap();
    // spell-checker:enable

    // check if they are valid
    let mut to_remove: Vec<usize> = Vec::new();
    for i in 0..paths.len() {
        if valid_file_extensions.is_match(&paths[i].filename) == false || extra_videos.is_match(&paths[i].filename) == true {
            to_remove.push(i);
        }
    }
    
    // remove in reverse order because values before index won't move but values after index will move
    to_remove.sort();
    to_remove.reverse();
    for r in to_remove {
        paths.remove(r);
    }

    paths.iter_mut().for_each(|path| {
        path.filename = valid_file_extensions.replace_all(&path.filename, "").to_string();
    })
}

// compares filename to anime titles using multiple string matching algorithms and remembers the most similar title
async fn string_similarity(paths: &mut Vec<AnimePathWorking>, media_id: Option<i32>) {

    let mut previous_file_name = String::new();
    let anime_data = GLOBAL_ANIME_DATA.lock().await.clone();

    // let mut folders = paths.first().unwrap().path.split("\\");
    // let index = folders.clone().count();
    // let folder = format!("data/{}_string_similarity.txt",folders.nth(index-2).unwrap());
    // let path = Path::new(folder.as_str());

    // if path.exists() {
    //     match fs::remove_file(path) {
    //         Err(why) => panic!("unable to remove, {}", why),
    //         Ok(file) => file,
    //     };
    // }

    // // create the file
    // let mut file = match File::create(path) {
    //     Err(why) => panic!("unable to open, {}", why),
    //     Ok(file) => file,
    // };

    paths.iter_mut().for_each(|path| {
        // skip files that have the same title
        if path.filename == previous_file_name {
            return;
        }
        else {
            previous_file_name = path.filename.clone();
        }

        let (id, _title, similarity_score) = identify_media_id(&path.filename, &anime_data, media_id);
        // match file.write_all(format!("{} | {} | {}\n", similarity_score, path.filename, title).as_bytes()) {
        //     Err(why) => panic!("ERROR: {}", why),
        //     Ok(file) => file,
        // };
        
        if similarity_score > 0.8 && similarity_score > path.similarity_score {
            path.media_id = id;
            path.similarity_score = similarity_score;
        }
    });

    // fill in data for files that were skipped
    for i in 1..paths.len() {
        if paths[i].filename == paths[i - 1].filename {
            paths[i].similarity_score = paths[i - 1].similarity_score;
            paths[i].media_id = paths[i - 1].media_id;
        }
    }

}

// returns the media id and similarity score based on the title
pub fn identify_media_id(filename: &String, anime_data: &HashMap<i32,AnimeInfo>, only_compare: Option<i32>) -> (i32, String, f64) {

    let mut score = 0.0;
    let mut media_id = 0;
    let mut title = String::new();
    
    if only_compare.is_none() {

        anime_data.iter().for_each(|data| {
            
            title_compare(data.1, filename, &mut score, &mut media_id, &mut title);
        });
    } else if anime_data.contains_key(&only_compare.unwrap()) {
        
        let anime = anime_data.get(&only_compare.unwrap()).unwrap();
        title_compare(anime, filename, &mut score, &mut media_id, &mut title);
    }
    (media_id, title, score)
}


fn title_compare(anime: &AnimeInfo, filename: &String, score: &mut f64, media_id: &mut i32, return_title: &mut String) {
    
    let mut titles: Vec<String> = Vec::new();
    if anime.title.english.is_some() { titles.push(anime.title.english.clone().unwrap().to_ascii_lowercase()) }
    if anime.title.romaji.is_some() { titles.push(anime.title.romaji.clone().unwrap().to_ascii_lowercase()) }
    if anime.title.native.is_some() { titles.push(anime.title.native.clone().unwrap().to_ascii_lowercase()) }

    for title in titles {

        if title.chars().next().unwrap() != filename.chars().next().unwrap() { continue } // skip comparison if first character does not match
        let normalized_levenshtein_score = strsim::normalized_levenshtein(&filename, &title);
        if normalized_levenshtein_score > *score { 
            *media_id = anime.id; 
            *score = normalized_levenshtein_score;
            *return_title = title;
        }
    }
}

// find the episode number in the filename and store it
fn identify_episode_number(paths: &mut Vec<AnimePathWorking>) {

    paths.iter_mut().for_each(|name| {

        let episode = identify_number(&name.filename);
        if episode.1 != 0 {
            name.episode = episode.1;
            name.filename = name.filename.replace(episode.0.as_str(), "");
        }
    });
}

// applies multiple regex to find the episode number
pub fn identify_number(filename: &String) -> (String, i32) {

    // remove episode titles with numbers that would be misidentified as episode numbers
    let episode_title_number = Regex::new(r"'.*\d+.*'").unwrap();
    let filename_episode_title_removed = episode_title_number.replace_all(&filename, "").to_string();

    // most anime fit this format
    let num1 = extract_number(&filename_episode_title_removed, Regex::new(r" - (\d+)").unwrap());
    if num1.1 != 0 {
        return num1;
    }
    // less common formats
    let num2 = extract_number(&filename_episode_title_removed, Regex::new(r" - Episode (\d+)").unwrap());
    if num2.1 != 0 {
        return num2;
    }
    let num3 = extract_number(&filename_episode_title_removed, Regex::new(r"[eE][pP] ?(\d+)").unwrap());
    if num3.1 != 0 {
        return num3;
    }
    // wider search for numbers, use last number that is not a version or season number
    let num4 = extract_number(&filename_episode_title_removed, Regex::new(r"[^vsVS](\d+)").unwrap());
    if num4.1 != 0 {
        return num4;
    }

    (String::new(), 0)
}

// finds and returns the episode number and wider string according to the regex rules
fn extract_number(filename: &String, regex: Regex) -> (String, i32) {

    let last_match = regex.find_iter(&filename).last();
    if last_match.is_none() { 
        return (String::new(),0)
    }

    let episode = last_match.unwrap().as_str();
    let captures = regex.captures(episode).unwrap();
    (episode.to_string(), captures.get(1).unwrap().as_str().parse().unwrap())
}


// write episode number found into a file
/*
fn file_dump_episode(paths: &mut Vec<AnimePathWorking>) {
    
    let path = Path::new("data/episode_data.txt");
    let mut file: File;
    // create the file
    file = match File::create(path) {
        Err(why) => panic!("unable to open, {}", why),
        Ok(file) => file,
    };

    paths.iter().for_each(|name| { 
        
        match write!(&mut file, "\n{} {}",name.filename, name.episode) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
    });
}
*/


fn irrelevant_information_removal_paths(paths: &mut Vec<AnimePathWorking>) {
    
    paths.iter_mut().for_each(|name| {

        name.filename = irrelevant_information_removal(name.filename.clone());
    });
}

// regex used to filter out useless information
lazy_static! {
    static ref VERSION: Regex = Regex::new(r"[vV]\d+").unwrap();
    static ref TRAILING_SPACES: Regex = Regex::new(r" +$").unwrap();
    static ref TRAILING_DASH: Regex = Regex::new(r" - $").unwrap();
    static ref TRAILING_DASH2: Regex = Regex::new(r" -$").unwrap();
    static ref DOTS_AS_SPACES: Regex = Regex::new(r"\w\.\w").unwrap();
    static ref EPISODE_TITLE: Regex = Regex::new(r"'.+'").unwrap();
    static ref XVID: Regex = Regex::new(r"[xX][vV][iI][dD]").unwrap();
}
// remove any extra information that will interfere with comparing the filename with the anime title
pub fn irrelevant_information_removal(filename: String) -> String {
    
    // replace underscores with spaces to increase similarity with titles
    let mut filename_clean = filename.replace("_", " ");

    // replace dots with spaces to increase similarity with titles
    if DOTS_AS_SPACES.is_match(&filename) {
        filename_clean = filename_clean.replace(".", " ");
    }

    // remove extra information that is not part of the title
    filename_clean = filename_clean.replace("dvd", "")
        .replace("DVD", "")
        .replace("Remastered", "")
        .replace("remastered", "")
        .replace(" Episode", "")
        .replace(" Ep", "")
        .replace(" EP", "")
        .replace(" E ", "")
        .replace(" END", "")
        .replace(" FINAL", "");

    filename_clean = VERSION.replace_all(&filename_clean, "").to_string();
    filename_clean = XVID.replace_all(&filename_clean, "").to_string();
    filename_clean = TRAILING_DASH.replace_all(&filename_clean, "").to_string();
    filename_clean = TRAILING_DASH2.replace_all(&filename_clean, "").to_string();
    filename_clean = EPISODE_TITLE.replace_all(&filename_clean, "").to_string();
    filename_clean = TRAILING_SPACES.replace_all(&filename_clean, "").to_string();
    filename_clean = TRAILING_DASH2.replace_all(&filename_clean, "").to_string();

    // convert title to lowercase so the comparison doesn't think upper/lower case letters are different
    filename_clean.to_ascii_lowercase()
}

pub fn extract_resolution(title: &String) -> i32 {

    let resolution_1080 = Regex::new(r"1080p|1080|1920x1080").unwrap();
    if resolution_1080.is_match(title) {
        return 1080;
    }

    let resolution_720 = Regex::new(r"720p|720|960x720|1280x720").unwrap();
    if resolution_720.is_match(title) {
        return 720;
    }

    let resolution_480 = Regex::new(r"480p|480|720x480|852x480").unwrap();
    if resolution_480.is_match(title) {
        return 480;
    }

    let resolution_other = Regex::new(r"\d\d\d\d?x(\d\d\d\d?)").unwrap();
    if resolution_other.is_match(title) {
        let captures = resolution_other.captures(title).unwrap();
        return captures.get(1).unwrap().as_str().parse().unwrap()
    }

    let resolution_other2 = Regex::new(r"(\d\d\d\d?)p").unwrap();
    if resolution_other2.is_match(title) {
        let captures = resolution_other2.captures(title).unwrap();
        return captures.get(1).unwrap().as_str().parse().unwrap()
    }

    let dvd = Regex::new(r"[Dd][Vv][Dd]").unwrap();
    if dvd.is_match(title) {
        return 480;
    }

    0
}


pub fn extract_sub_group(title: &String) -> String {

    let sub_group_find = Regex::new(r"^\[([^\[\]]+)\]").unwrap();
    if sub_group_find.is_match(title) {
        let captures = sub_group_find.captures(title).unwrap();
        return captures.get(1).unwrap().as_str().to_string();
    }

    return String::new()
}