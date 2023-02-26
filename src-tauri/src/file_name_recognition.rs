use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;
use regex::Regex;
use serde::{Serialize, Deserialize};


use crate::api_calls::{AnimeInfo, self};
use crate::{GLOBAL_ANIME_DATA, GLOBAL_ANIME_PATH, GLOBAL_USER_SETTINGS, file_operations, GLOBAL_REFRESH_UI};
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
    
    let mut timer = Instant::now();
    
    get_prequel_data().await;
    
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
                    let file_name = unwrap.file_name().to_str().unwrap().to_string();
                    let path = unwrap.path().to_str().unwrap().to_string();
                    file_names.push(AnimePathWorking::new(path, file_name));
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

        let anime_data = GLOBAL_ANIME_DATA.lock().await.clone();

        string_similarity(&mut file_names, media_id, &anime_data);
        
        replace_with_sequel_batch(&mut file_names, &anime_data);

        episode_fix_batch(&mut file_names, &anime_data);
        
        let mut file_paths = GLOBAL_ANIME_PATH.lock().await;
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
    GLOBAL_REFRESH_UI.lock().await.canvas = true;
    file_operations::write_file_episode_path().await;
    println!("parse_file_names finished");
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
fn string_similarity(paths: &mut Vec<AnimePathWorking>, media_id: Option<i32>, anime_data: &HashMap<i32, AnimeInfo>) {

    let mut previous_file_name = String::new();

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

        if score < 0.7 {

            let pre_dash = Regex::new(r"([^-]*).*").unwrap();
            let captures = pre_dash.captures(filename);
            if captures.is_some() {

                let modified_filename = captures.unwrap().get(1).unwrap().as_str().to_string();
                anime_data.iter().for_each(|data| {
                    
                    title_compare(data.1, &modified_filename, &mut score, &mut media_id, &mut title);
                });
            }
        }

/*         let season_shortened = Regex::new(r"[sS](\d+)").unwrap();
        if season_shortened.is_match(filename) {

            let mut season_number = season_shortened.captures(&filename).unwrap().get(1).unwrap().as_str().to_string();
            season_number.insert_str(0, "season ");
            let modified_filename = season_shortened.replace(&filename, season_number).to_string();

            anime_data.iter().for_each(|data| {
                
                title_compare(data.1, &modified_filename, &mut score, &mut media_id, &mut title);
            });
        } */

    } else if anime_data.contains_key(&only_compare.unwrap()) {
        
        let anime = anime_data.get(&only_compare.unwrap()).unwrap();
        title_compare(anime, filename, &mut score, &mut media_id, &mut title);
    }
    (media_id, title, score)
}


fn title_compare(anime: &AnimeInfo, filename: &String, score: &mut f64, media_id: &mut i32, return_title: &mut String) {
    
    if filename.len() == 0 {
        return;
    }

    let mut titles: Vec<String> = Vec::new();
    if anime.title.english.is_some() { titles.push(replace_special_vowels(anime.title.english.clone().unwrap().to_ascii_lowercase())) }
    if anime.title.romaji.is_some() { titles.push(replace_special_vowels(anime.title.romaji.clone().unwrap().to_ascii_lowercase())) }
    if anime.title.native.is_some() { titles.push(replace_special_vowels(anime.title.native.clone().unwrap().to_ascii_lowercase())) }

    for title in titles {

        //if title.chars().next().unwrap() != filename.chars().next().unwrap() { continue } // skip comparison if first character does not match
        let no_special_vowels_filename = replace_special_vowels(filename.to_ascii_lowercase());
        let normalized_levenshtein_score = strsim::normalized_levenshtein(&no_special_vowels_filename, &title);
        if normalized_levenshtein_score > *score { 
            *media_id = anime.id; 
            *score = normalized_levenshtein_score;
            *return_title = title.clone();
        }
    }
}



lazy_static! {
    static ref REPLACE_A: Regex = Regex::new(r"À|Á|Â|Ã|Ä|Å|à|á|â|ã|ä|å").unwrap();
    static ref REPLACE_AE: Regex = Regex::new(r"Æ|æ").unwrap();
    static ref REPLACE_C: Regex = Regex::new(r"Ç|ç").unwrap();
    static ref REPLACE_E: Regex = Regex::new(r"È|É|Ê|Ë|è|é|ê|ë").unwrap();
    static ref REPLACE_I: Regex = Regex::new(r"Ì|Í|Î|Ï|ì|í|î|ï").unwrap();
    static ref REPLACE_D: Regex = Regex::new(r"Ð|ð").unwrap();
    static ref REPLACE_N: Regex = Regex::new(r"Ñ|ñ").unwrap();
    static ref REPLACE_O: Regex = Regex::new(r"Ò|Ó|Ô|Õ|Ö|Ø|ò|ó|ô|õ|ö|ø").unwrap();
    static ref REPLACE_U: Regex = Regex::new(r"Ù|Ú|Û|Ü|ù|ú|û|ü").unwrap();
    static ref REPLACE_Y: Regex = Regex::new(r"Ý|ý|ÿ").unwrap();
    static ref REPLACE_B: Regex = Regex::new(r"ß|Þ|þ").unwrap();
}
// replaces vowels with special marks
// most people don't have these characters on their keyboard so they may create a discrepancy between the filename and official title
fn replace_special_vowels(text: String) -> String {

    let mut result = REPLACE_A.replace_all(&text, "a").to_string();
    result = REPLACE_AE.replace_all(&result, "ae").to_string();
    result = REPLACE_C.replace_all(&result, "c").to_string();
    result = REPLACE_E.replace_all(&result, "e").to_string();
    result = REPLACE_I.replace_all(&result, "i").to_string();
    result = REPLACE_D.replace_all(&result, "d").to_string();
    result = REPLACE_N.replace_all(&result, "n").to_string();
    result = REPLACE_O.replace_all(&result, "o").to_string();
    result = REPLACE_U.replace_all(&result, "u").to_string();
    result = REPLACE_Y.replace_all(&result, "y").to_string();
    result = REPLACE_B.replace_all(&result, "b").to_string();

    result
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
// returns the text containing episode information, the episode, and the number of episodes in the file
pub fn identify_number(filename: &String) -> (String, i32, i32) {

    let captures = Regex::new(r"[^sS](\d+)[&-](\d+)").unwrap().captures(filename);
    if captures.is_some() {
        let captures2 = captures.unwrap();
        let episode = captures2.get(1).unwrap().as_str().parse().unwrap();
        let length = 1 + captures2.get(2).unwrap().as_str().parse::<i32>().unwrap() - episode;
        return (captures2.get(0).unwrap().as_str().to_string(), episode, length);
    }

    // remove episode titles with numbers that would be misidentified as episode numbers
    let episode_title_number = Regex::new(r"'.*\d+.*'").unwrap();
    let filename_episode_title_removed = episode_title_number.replace_all(&filename, "").to_string();

    // most anime fit this format
    let num1 = extract_number(&filename_episode_title_removed, Regex::new(r" - (\d+)").unwrap());
    if num1.1 != 0 {
        return (num1.0, num1.1, 1);
    }
    // less common formats
    let num2 = extract_number(&filename_episode_title_removed, Regex::new(r" - Episode (\d+)").unwrap());
    if num2.1 != 0 {
        return (num2.0, num2.1, 1);
    }
    let num3 = extract_number(&filename_episode_title_removed, Regex::new(r"[eE][pP] ?(\d+)").unwrap());
    if num3.1 != 0 {
        return (num3.0, num3.1, 1);
    }
    // wider search for numbers, use last number that is not a version or season number
    let num4 = extract_number(&filename_episode_title_removed, Regex::new(r"[^vsVS](\d+)").unwrap());
    if num4.1 != 0 {
        return (num4.0, num4.1, 1);
    }

    (String::new(), 0, 0)
}


// finds and returns the episode number and wider string according to the regex rules
fn extract_number(filename: &String, regex: Regex) -> (String, i32) {

    let last_match = regex.find_iter(&filename).last();
    // no number found
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

    let dvd = Regex::new(r"([Dd][Vv][Dd])|[Ss][Dd]").unwrap();
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

// replace a anime with its sequel if the episode number is too high
fn replace_with_sequel_batch(paths: &mut Vec<AnimePathWorking>, anime_data: &HashMap<i32, AnimeInfo>) {

    for path in paths {

        (path.media_id, path.episode) = replace_with_sequel(path.media_id, path.episode, &anime_data);
    }
}

pub fn replace_with_sequel(mut anime_id: i32, mut episode: i32, anime_data: &HashMap<i32, AnimeInfo>) -> (i32, i32) {

    // anime is not in list or anime has unknown number of episodes which means it has no sequels
    if anime_data.contains_key(&anime_id) == false || anime_data.get(&anime_id).unwrap().episodes.is_none() {
        return (anime_id, episode);
    }

    // episode is within episode count
    let mut episodes = anime_data.get(&anime_id).unwrap().episodes.unwrap();
    if episode <= episodes {
        return (anime_id, episode);
    }

    // start from the first season
    let mut prequel_exists = true;
    while prequel_exists {

        prequel_exists = false;
        for edge in anime_data.get(&anime_id).unwrap().relations.edges.iter() {

            if edge.relation_type == "PREQUEL" && anime_data.contains_key(&edge.node.id) && anime_data.get(&edge.node.id).unwrap().format.as_ref().unwrap() == "TV" {
                anime_id = edge.node.id;
                episodes = anime_data.get(&anime_id).unwrap().episodes.unwrap();
                prequel_exists = true;
            }
        }
    }

    // traverse across sequels until episode is within episode count
    let mut sequel_exists = true;
    while episode > episodes && sequel_exists {

        sequel_exists = false;
        for edge in anime_data.get(&anime_id).unwrap().relations.edges.iter() {
            if edge.relation_type == "SEQUEL" && anime_data.contains_key(&edge.node.id) && anime_data.get(&edge.node.id).unwrap().format.as_ref().unwrap() == "TV" {
                anime_id = edge.node.id;
                episode -= episodes;
                sequel_exists = true;
                break;
            }
        }
        if anime_data.get(&anime_id).unwrap().episodes.is_none() {
            break;
        }
        episodes = anime_data.get(&anime_id).unwrap().episodes.unwrap();
    }

    (anime_id, episode)
}


// get anime data for prequels of any anime that is in anime data global
// necessary for recognizing anime that is labeled as one anime but belongs to a sequel of that anime
// for example boku no hero academia episode 100 when no season has 100 episodes
pub async fn get_prequel_data() {

    let anime_data = GLOBAL_ANIME_DATA.lock().await;
    let mut get_info: Vec<i32> = Vec::new();

    for (_, anime) in anime_data.iter() {

        for edge in anime.relations.edges.iter() {

            if edge.relation_type == "PREQUEL" && edge.node.media_type == "ANIME" && anime_data.contains_key(&edge.node.id) == false {

                get_info.push(edge.node.id);
                println!("{} {}", anime.title.romaji.as_ref().unwrap(), edge.node.id);
            }
        }
    }
    drop(anime_data);

    while get_info.is_empty() == false {
        println!("get_info size {}", get_info.len());
        api_calls::anilist_api_call_multiple(get_info.clone()).await;
        let anime_ids = get_info.clone();
        get_info.clear();
        let anime_data = GLOBAL_ANIME_DATA.lock().await;
        for id in anime_ids {

            if anime_data.contains_key(&id) == false {

                continue;
            }
            for edge in anime_data.get(&id).unwrap().relations.edges.iter() {

                if edge.relation_type == "PREQUEL" && anime_data.contains_key(&edge.node.id) == false {
    
                    get_info.push(edge.node.id);
                }
            }
        }
        drop(anime_data);
    }
    file_operations::write_file_anime_info_cache().await;
}


fn episode_fix_batch(paths: &mut Vec<AnimePathWorking>, anime_data: &HashMap<i32, AnimeInfo>) {

    paths.iter_mut().for_each(|entry| {

        episode_fix(entry.media_id, &mut entry.episode, &anime_data);
    });
}

// will fix the episode number for numbers in titles of movies, ova's, etc
pub fn episode_fix(anime_id: i32, episode: &mut i32, anime_data: &HashMap<i32, AnimeInfo>) {

    let anime = anime_data.get(&anime_id);
    if anime.is_some() {

        let episodes = anime.unwrap().episodes;
        if episodes.is_some() {

            if episodes.unwrap() == 1 {

                *episode = 1;
            }
        }
    }
}