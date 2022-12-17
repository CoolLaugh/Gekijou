use std::fmt::format;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use regex::Regex;


use crate::GLOBAL_ANIME_DATA;
use similar_string::*;
use strsim;

// working struct to store data while determining what anime a file belongs to
#[derive(Debug, Clone, Default)]
struct AnimePathWorking {
    path: String,
    filename: String,
    episode: i32,
    media_id: i32,

    jaro: f64,
    jaro_id: i32,
    jaro_name: String,
    sorensen_dice: f64,
    sorensen_dice_id: i32,
    sorensen_dice_name: String,
    normalized_levenshtein: f64,
    normalized_levenshtein_id: i32,
    normalized_levenshtein_name: String,
    normalized_damerau_levenshtein: f64,
    normalized_damerau_levenshtein_id: i32,
    normalized_damerau_levenshtein_name: String,
}

impl AnimePathWorking {
    pub const fn new(new_path: String, new_filename: String) -> AnimePathWorking {
        AnimePathWorking { path: new_path, filename: new_filename, episode: 0, media_id: 0, normalized_levenshtein: 0.0, normalized_levenshtein_name: String::new(), normalized_damerau_levenshtein: 0.0, normalized_damerau_levenshtein_name: String::new(), jaro: 0.0, jaro_name: String::new(), sorensen_dice: 0.0, sorensen_dice_name: String::new(), normalized_damerau_levenshtein_id: 0, jaro_id: 0, sorensen_dice_id: 0, normalized_levenshtein_id: 0 }
    }
}

pub async fn parse_file_names(folders: Vec<String>) {

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
            name.filename = Regex::new(r"((\[[^\[\]]+\]|\([^\(\)]+\))[ _]*)+").unwrap().replace_all(&name.filename, "").to_string();
        });

        identify_episode_number(&mut file_names);

        //file_dump_episode(&mut file_names);

        irrelevant_information_removal(&mut file_names);

        let now = Instant::now();
        string_similarity(&mut file_names).await;
        let elapsed_time = now.elapsed();
        println!("Took {} seconds.", elapsed_time.as_secs());
        
        file_dump_formatted(&mut file_names);
    }
}



// removes any files that are the wrong file type or extra (openings, endings, etc)
fn remove_invalid_files(paths: &mut Vec<AnimePathWorking>) {

    // remove files that are not video files
    let valid_file_extentions = Regex::new(r"[_ ]?(\.mkv|\.avi|\.mp4)").unwrap();
    // remove openings, endings, PV, and other non episode videos
    let extra_videos = Regex::new(r"[ _\.][oO][pP]\d*([vV]\d)?[ _\.]|[ _\.]NCOP\d*([vV]\d)?[ _\.]|[ _\.]NCED\d*([vV]\d)?[ _\.]|[ _\.][eE][dD]\d*([vV]\d)?[ _\.]|[ _\.][sS]kit[ _\.]|[eE]nding|[oO]pening|[ _][pP][vV][ _]|[bB][dD] [mM][eE][nN][uU]").unwrap();

    // check if they are valid
    let mut to_remove: Vec<usize> = Vec::new();
    for i in 0..paths.len() {
        if valid_file_extentions.is_match(&paths[i].filename) == false || extra_videos.is_match(&paths[i].filename) == true {
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
        path.filename = valid_file_extentions.replace_all(&path.filename, "").to_string();
    })
}



// compares filename to anime titles using multiple string matching algorithms and remembers the most similar title
async fn string_similarity(paths: &mut Vec<AnimePathWorking>) {

    let anime_data = GLOBAL_ANIME_DATA.lock().await;
    let mut previous_file_name = String::new();
    let mut counter = 0;
    let total = paths.len();

    paths.iter_mut().for_each(|path| {
        // skip files that have the same title
        counter += 1;
        println!("{}/{}", counter, total);
        if path.filename == previous_file_name {
            return;
        }
        else {
            previous_file_name = path.filename.clone();
        }

        anime_data.iter().for_each(|data| {
            
            let mut titles: Vec<String> = Vec::new();
            if data.1.title.english.is_some() { titles.push(data.1.title.english.clone().unwrap().to_ascii_lowercase()) }
            if data.1.title.romaji.is_some() { titles.push(data.1.title.romaji.clone().unwrap().to_ascii_lowercase()) }
            if data.1.title.native.is_some() { titles.push(data.1.title.native.clone().unwrap().to_ascii_lowercase()) }

            for title in titles {

                let normalized_levenshtein_score = strsim::normalized_levenshtein(&path.filename, &title);
                if normalized_levenshtein_score > path.normalized_levenshtein { path.normalized_levenshtein_name = title.clone(); path.normalized_levenshtein = normalized_levenshtein_score; }

                let normalized_damerau_levenshtein_score = strsim::normalized_damerau_levenshtein(&path.filename, &title);
                if normalized_damerau_levenshtein_score > path.normalized_damerau_levenshtein { path.normalized_damerau_levenshtein_name = title.clone(); path.normalized_damerau_levenshtein = normalized_damerau_levenshtein_score; }

                let jaro_score = strsim::jaro(&path.filename, &title);
                if jaro_score > path.jaro { path.jaro_name = title.clone(); path.jaro = jaro_score; }

                let sorensen_dice_score = strsim::sorensen_dice(&path.filename, &title);
                if sorensen_dice_score > path.sorensen_dice { path.sorensen_dice_name = title; path.sorensen_dice = sorensen_dice_score; }
            }
        });
    });

    // fill in data for files that were skipped
    for i in 0..paths.len() {
        if i == 0 { continue; }
        if paths[i].normalized_levenshtein == 0.0 {
            paths[i].normalized_levenshtein = paths[i - 1].normalized_levenshtein;
            paths[i].normalized_damerau_levenshtein = paths[i - 1].normalized_damerau_levenshtein;
            paths[i].jaro = paths[i - 1].jaro;
            paths[i].sorensen_dice = paths[i - 1].sorensen_dice;
            paths[i].normalized_levenshtein_name = paths[i - 1].normalized_levenshtein_name.clone();
            paths[i].normalized_damerau_levenshtein_name = paths[i - 1].normalized_damerau_levenshtein_name.clone();
            paths[i].jaro_name = paths[i - 1].jaro_name.clone();
            paths[i].sorensen_dice_name = paths[i - 1].sorensen_dice_name.clone();
        }
    }

}


// find the episode number in the filename and store it
fn identify_episode_number(paths: &mut Vec<AnimePathWorking>) {

    // remove episode titles with numbers that would be misidentified as episode numbers
    let episode_title_number = Regex::new(r"'.*\d+.*'").unwrap();
    paths.iter_mut().for_each(|name| {
        name.filename = episode_title_number.replace_all(&name.filename, "").to_string();
    });

    // most anime fit this format
    extract_number(paths, Regex::new(r" - (\d+)").unwrap());

    // less common formats
    extract_number(paths, Regex::new(r" - Episode (\d+)").unwrap());
    extract_number(paths, Regex::new(r"[eE][pP] ?(\d+)").unwrap());

    // wider search for numbers, use last number that is not a version or season number
    extract_number(paths, Regex::new(r"[^vsVS](\d+)").unwrap());
}

fn extract_number(paths: &mut Vec<AnimePathWorking>, regex: Regex) {

    paths.iter_mut().for_each(|name| {

        if name.episode != 0 {
            return;
        }

        let last_match = regex.find_iter(&name.filename).last();
        if last_match.is_none() { 
            return;
        }

        let episode = last_match.unwrap().as_str();
        let captures = regex.captures(episode).unwrap();
        name.episode = captures.get(1).unwrap().as_str().parse().unwrap();
        name.filename = name.filename.replace(episode, "");
    });
}

// write episode number found into a file
fn file_dump_episode(paths: &mut Vec<AnimePathWorking>) {
    
    let path = Path::new("episode_data.txt");
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

// write results into a file
fn file_dump_formatted(paths: &mut Vec<AnimePathWorking>) {
    
    let path = Path::new("match_data.txt");
    let mut file: File;
    // create the file
    file = match File::create(path) {
        Err(why) => panic!("unable to open, {}", why),
        Ok(file) => file,
    };

    match write!(&mut file, "--------------------------------------------------------------------------------------------------------------------------------") {
        Err(why) => panic!("ERROR: {}", why),
        Ok(file) => file,
    };

    let mut previous_title = String::new();

    paths.iter().for_each(|name| { 
        
        if name.filename == previous_title {
            return;
        } else {
            previous_title = name.filename.clone();
        }

        match write!(&mut file, "\nnormalized_levenshtein:\t\t\t | {:.4} | {} | {}", name.normalized_levenshtein, name.filename, name.normalized_levenshtein_name) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
        
        match write!(&mut file, "\nnormalized_damerau_levenshtein:  | {:.4} | {} | {}", name.normalized_damerau_levenshtein, name.filename, name.normalized_damerau_levenshtein_name) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
        
        match write!(&mut file, "\njaro:\t\t\t\t\t\t\t | {:.4} | {} | {}", name.jaro, name.filename, name.jaro_name) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
        
        match write!(&mut file, "\nsorensen_dice:\t\t\t\t\t | {:.4} | {} | {}", name.sorensen_dice, name.filename, name.sorensen_dice_name) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };

        let scores = vec![name.normalized_levenshtein, name.normalized_damerau_levenshtein, name.jaro, name.sorensen_dice];

        match write!(&mut file, "\nAverage:\t\t\t\t\t\t | {:.4} |", (scores[0] + scores[1] + scores[2] + scores[3]) / 4.0) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };

        let mut highest = 0.0;
        for num in scores.clone() {
            if num > highest { highest = num};
        }

        let mut lowest = 1.0;
        for num in scores {
            if num < lowest { lowest = num};
        }
            
        match write!(&mut file, "\nDistance:\t\t\t\t\t\t | {:.4} |", highest - lowest) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };

    match write!(&mut file, "\n--------------------------------------------------------------------------------------------------------------------------------") {
        Err(why) => panic!("ERROR: {}", why),
        Ok(file) => file,
    };
    });
}

fn irrelevant_information_removal(paths: &mut Vec<AnimePathWorking>) {
    
    let version = Regex::new(r"[vV]\d+").unwrap();
    let trailing_spaces = Regex::new(r" +$").unwrap();
    let trailing_dash = Regex::new(r" - $").unwrap();
    let trailing_dash2 = Regex::new(r" -$").unwrap();
    let dots_as_spaces = Regex::new(r"\w\.\w").unwrap();
    let episode_title = Regex::new(r"'.+'").unwrap();
    let xvid = Regex::new(r"[xX][vV][iI][dD]").unwrap();
    paths.iter_mut().for_each(|name| {

        // replace underscores with spaces to increase similarity with titles
        name.filename = name.filename.replace("_", " ");
        
        // replace dots with spaces to increase similarity with titles
        if dots_as_spaces.is_match(&name.filename) {
            name.filename = name.filename.replace(".", " ");
        }

        // remove extra information that is not part of the title
        name.filename = name.filename.replace("dvd", "");
        name.filename = name.filename.replace("DVD", "");
        name.filename = name.filename.replace("Remastered", "");
        name.filename = name.filename.replace("remastered", "");
        name.filename = name.filename.replace(" Episode", "");
        name.filename = name.filename.replace(" Ep", "");
        name.filename = name.filename.replace(" EP", "");
        name.filename = name.filename.replace(" E ", "");
        name.filename = name.filename.replace(" END", "");
        name.filename = name.filename.replace(" FINAL", "");

        name.filename = version.replace_all(&name.filename, "").to_string();
        name.filename = xvid.replace_all(&name.filename, "").to_string();
        name.filename = trailing_dash.replace_all(&name.filename, "").to_string();
        name.filename = trailing_dash2.replace_all(&name.filename, "").to_string();
        name.filename = episode_title.replace_all(&name.filename, "").to_string();
        name.filename = trailing_spaces.replace_all(&name.filename, "").to_string();
        name.filename = trailing_dash2.replace_all(&name.filename, "").to_string();

        // convert title to lowercase so the comparison doesn't think upper/lower case letters are different
        name.filename = name.filename.to_ascii_lowercase();
    });
}