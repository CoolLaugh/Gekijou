use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use regex::Regex;


use crate::api_calls::AnimeInfo;
use crate::{GLOBAL_ANIME_DATA, GLOBAL_ANIME_PATH};
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

pub struct AnimePath {
    pub path: String,
    pub similarity_score: f64,
}

// scans these folders and subfolders looking for files that match titles in the users anime list
// found files are then stored in a global list for each anime and episode
pub async fn parse_file_names(folders: &Vec<String>) {
    
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
        
        let timer = Instant::now();

        string_similarity(&mut file_names).await;
        
        let millis = timer.elapsed().as_millis();
        println!("Time: {}s {}ms", millis / 1000, millis % 1000);
        
        let mut file_paths = GLOBAL_ANIME_PATH.lock().await;

        for file in file_names {

            if file.similarity_score > 0.8 {

                let media = file_paths.entry(file.media_id).or_default();
                if media.contains_key(&file.episode) && media.get(&file.episode).unwrap().similarity_score < file.similarity_score {
                    media.entry(file.media_id).and_modify(|anime_path| {

                        anime_path.similarity_score = file.similarity_score;
                        anime_path.path = file.path;
                    });
                } else {

                    media.insert(file.episode, AnimePath { path: file.path, similarity_score: file.similarity_score });
                }
            }
        }
    }
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
async fn string_similarity(paths: &mut Vec<AnimePathWorking>) {

    //let anime_data = GLOBAL_ANIME_DATA.lock().await;
    let mut previous_file_name = String::new();
    //let mut counter = 0;
    //let total = paths.len();
    let anime_data = GLOBAL_ANIME_DATA.lock().await;

    paths.iter_mut().for_each(|path| {
        // skip files that have the same title
        //counter += 1;
        //println!("{}/{}", counter, total);
        if path.filename == previous_file_name {
            return;
        }
        else {
            previous_file_name = path.filename.clone();
        }

        let similarity_score = identify_media_id(&path.filename, &anime_data);
        
        if similarity_score.1 > 0.8 {
            path.media_id = similarity_score.0;
            path.similarity_score = similarity_score.1;
        }

    });

    // fill in data for files that were skipped
    for i in 0..paths.len() {
        if i == 0 { continue; }
        if paths[i].media_id == 0 {
            paths[i].similarity_score = paths[i - 1].similarity_score;
            paths[i].media_id = paths[i - 1].media_id;
        }
    }

}

// returns the media id and similarity score based on the title
pub fn identify_media_id(filename: &String, anime_data: &HashMap<i32,AnimeInfo>) -> (i32, f64) {

    let mut score = 0.0;
    let mut media_id = 0;
    anime_data.iter().for_each(|data| {
            
        let mut titles: Vec<String> = Vec::new();
        if data.1.title.english.is_some() { titles.push(data.1.title.english.clone().unwrap().to_ascii_lowercase()) }
        if data.1.title.romaji.is_some() { titles.push(data.1.title.romaji.clone().unwrap().to_ascii_lowercase()) }
        if data.1.title.native.is_some() { titles.push(data.1.title.native.clone().unwrap().to_ascii_lowercase()) }

        for title in titles {

            if title.chars().next().unwrap() != filename.chars().next().unwrap() { continue }
            let normalized_levenshtein_score = strsim::normalized_levenshtein(&filename, &title);
            if normalized_levenshtein_score > score { media_id = data.1.id; score = normalized_levenshtein_score; }
        }
    });
    (media_id, score)
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