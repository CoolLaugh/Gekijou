// use std::collections::HashMap;
// use std::sync::mpsc;
// use std::thread;
// use std::path::Path;
// use regex::Regex;
// use serde::{Serialize, Deserialize};
// use tauri::async_runtime::Mutex;
// use walkdir::WalkDir;
// use crate::{GLOBAL_ANIME_PATH, file_operations, GLOBAL_REFRESH_UI, constants, GLOBAL_KNOWN_FILES};
// use strsim;



// // working struct to store data while determining what anime a file belongs to
// #[derive(Debug, Clone, Default)]
// struct AnimePathWorking {
//     path: String,
//     filename: String,
//     episode: i32,
//     media_id: i32,
//     similarity_score: f64,
// }



// impl AnimePathWorking {
//     pub const fn new(new_path: String, new_filename: String) -> AnimePathWorking {
//         AnimePathWorking { path: new_path, filename: new_filename, episode: 0, media_id: 0, similarity_score: 0.0 }
//     }
// }



// #[derive(Serialize, Deserialize, Debug, Clone, Default)]
// pub struct AnimePath {
//     pub path: String,
//     pub similarity_score: f64,
// }

// lazy_static! {
//     static ref SCAN_MUTEX: Mutex<()> = Mutex::new(());
// }

// // scans these folders and subfolders looking for files that match titles in the users anime list
// // found files are then stored in a global list for each anime and episode
// // media_id is for finding files for a specific anime instead of any anime known
// pub async fn parse_file_names(skip_files: bool, media_id: Option<i32>) -> bool {

//     // don't start another scan while a scan is currently occurring
//     if GLOBAL_REFRESH_UI.lock().await.scan_data.current_folder > 0 && media_id.is_none() {
//         return false;
//     }

//     let _scan_lock = SCAN_MUTEX.lock().await;

//     get_prequel_data().await;

//     if skip_files == true && GLOBAL_KNOWN_FILES.lock().await.len() == 0 {
//         match file_operations::read_file_known_files(&GLOBAL_KNOWN_FILES).await {
//             Ok(_result) => { /* do nothing */ },
//             Err(_error) => { /* ignore */ },
//         }
//     }

//     let mut episode_found = false;
//     let folders = GLOBAL_USER_SETTINGS.lock().await.folders.clone();
//     GLOBAL_REFRESH_UI.lock().await.scan_data.clear();
//     GLOBAL_REFRESH_UI.lock().await.scan_data.total_folders = folders.len() as i32;
//     for folder in folders {

//         GLOBAL_REFRESH_UI.lock().await.scan_data.current_folder += 1;
//         GLOBAL_REFRESH_UI.lock().await.scan_data.completed_chunks = 0;
//         let path = Path::new(&folder);
//         if path.exists() == false {
//             continue;
//         }

//         let file_names: Vec<AnimePathWorking> = {
//             let mut file_names_temp = Vec::new();
//             let valid_extensions = ["mkv", "mp4", "avi"];

//             for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {

//                 if entry.file_type().is_file() == false { 
//                     continue; 
//                 }

//                 if let Some(ext) = entry.path().extension() {
//                     if valid_extensions.contains(&ext.to_str().unwrap_or_default()) == false {
//                         continue;
//                     }

//                     let path = entry.path().to_path_buf().to_str().unwrap().to_string();
//                     let file_name = path.split('\\').last().unwrap().to_string();
//                     file_names_temp.push(AnimePathWorking::new(path, file_name));
//                 }
//             }

//             file_names_temp
//         };

//         let file_names_chunks = file_names.chunks(crate::constants::FILENAME_CHUNKS).map(|chunk| chunk.to_vec()).collect::<Vec<_>>();
//         println!("Number of files: {}", file_names.len());
//         println!("Number of chunks: {}", file_names_chunks.len());

//         GLOBAL_REFRESH_UI.lock().await.scan_data.total_chunks = file_names_chunks.len() as i32;
//         GLOBAL_REFRESH_UI.lock().await.scan_data.completed_chunks = 0;

//         let (sender, receiver) = mpsc::channel::<bool>();

//         let mut children = vec![];
//         for mut file_name_chunk in file_names_chunks {

//             let anime_data = GLOBAL_ANIME_DATA.lock().await.clone();
//             let known_files = GLOBAL_KNOWN_FILES.lock().await.clone();

//             let sender_copy = sender.clone();
//             children.push(thread::spawn(move || -> Vec<AnimePathWorking> {
                
//                 if skip_files == true {
//                     file_name_chunk.retain(|anime_path| known_files.contains(&anime_path.filename) == false);
//                 }

//                 remove_invalid_files(&mut file_name_chunk);
        
//                 // remove brackets and their contents, the name and episode are unlikely to be here
//                 file_name_chunk.iter_mut().for_each(|name| {
//                     name.filename = remove_brackets(&name.filename);
//                 });
                
//                 identify_episode_number(&mut file_name_chunk);
                
//                 irrelevant_information_removal_paths(&mut file_name_chunk);

//                 string_similarity(&mut file_name_chunk, media_id, &anime_data);
        
//                 replace_with_sequel_batch(&mut file_name_chunk, &anime_data);
        
//                 episode_fix_batch(&mut file_name_chunk, &anime_data);

//                 match sender_copy.send(true) {
//                     Ok(_result) => {
//                         // do nothing
//                     },
//                     Err(_error) => println!("no receiver"),
//                 }

//                 file_name_chunk
//             }));
//         }

//         for _received in receiver {
//             let mut refresh_ui = GLOBAL_REFRESH_UI.lock().await;
//             refresh_ui.scan_data.completed_chunks += 1;
//             if refresh_ui.scan_data.completed_chunks >= refresh_ui.scan_data.total_chunks {
//                 break;
//             }
//         }

//         if skip_files == true {
//             let mut known_files = GLOBAL_KNOWN_FILES.lock().await;
//             for anime_path in file_names {
//                 known_files.insert(anime_path.filename);
//             }
//         }
        
//         let file_names_collected = children.into_iter().map(|c| c.join().unwrap()).flatten().collect::<Vec<_>>();
        
//         let anime_data = GLOBAL_ANIME_DATA.lock().await.clone();
        
//         let mut file_paths = GLOBAL_ANIME_PATH.lock().await;
//         for file in file_names_collected {

//             // non anime file
//             if file.media_id == 0 || file.similarity_score < constants::SIMILARITY_SCORE_THRESHOLD {
//                 continue;
//             }

//             let file_in_range = if let Some(anime_entry) = anime_data.get(&file.media_id) {
//                 if let Some(episodes) = anime_entry.episodes {
//                     file.episode <= episodes
//                 } else {
//                     true
//                 }
//             } else {
//                 println!("media_id: {}", file.media_id);
//                 assert!(false); // should never be this.  anime entry must exist if file variable has its media id
//                 false
//             };

//             if file_in_range {

//                 let media = file_paths.entry(file.media_id).or_default();
//                 if media.contains_key(&file.episode) && media.get(&file.episode).unwrap().similarity_score < file.similarity_score {
//                     media.entry(file.media_id).and_modify(|anime_path| {

//                         anime_path.similarity_score = file.similarity_score;
//                         anime_path.path = file.path;
//                     });
//                     episode_found = true;
//                 } else {

//                     media.insert(file.episode, AnimePath { path: file.path, similarity_score: file.similarity_score });
//                     episode_found = true;
//                 }
//             }
//         }
//     }

//     if skip_files == true {
//         file_operations::write_file_known_files(&*GLOBAL_KNOWN_FILES.lock().await).await;
//     }
//     GLOBAL_REFRESH_UI.lock().await.scan_data.clear();
//     remove_missing_files().await;
//     GLOBAL_REFRESH_UI.lock().await.canvas = true;
//     file_operations::write_file_episode_path().await;
//     println!("parse_file_names finished");
//     episode_found
// }



// // remove all files that no longer exist
// async fn remove_missing_files() {

//     let mut file_paths = GLOBAL_ANIME_PATH.lock().await;
//     for (_, anime) in file_paths.iter_mut() {

//         anime.retain(|_, episode| { Path::new(&episode.path).exists() });
//     }
//     file_paths.retain(|_,anime| { anime.len() > 0 });
// }



// // remove all brackets from a filename
// pub fn remove_brackets(filename: &String) -> String {
//     Regex::new(r"((\[[^\[\]]+\]|\([^\(\)]+\))[ _]*)+").unwrap().replace_all(&filename, "").to_string()
// }



// // removes any files that are the wrong file type or extra (openings, endings, etc)
// fn remove_invalid_files(paths: &mut Vec<AnimePathWorking>) {

//     // remove openings, endings, PV, and other non episode videos
//     // spell-checker:disable
//     let extra_videos = Regex::new(r"[ _\.][oO][pP]\d*([vV]\d)?[ _\.]|[ _\.]NCOP\d*([vV]\d)?[ _\.]|[ _\.]NCED\d*([vV]\d)?[ _\.]|[ _\.][eE][dD]\d*([vV]\d)?[ _\.]|[ _\.][sS]kit[ _\.]|[eE]nding|[oO]pening|[ _][pP][vV][ _]|[bB][dD] [mM][eE][nN][uU]").unwrap();
//     // spell-checker:enable

//     // check if they are valid
//     paths.retain(|path| { extra_videos.is_match(&path.filename) == false });

//     // remove file extension
//     paths.iter_mut().for_each(|path| {
//         path.filename = path.filename.rsplit_once('.').unwrap().0.to_string();
//     });
// }



// // compares filename to anime titles using multiple string matching algorithms and remembers the most similar title
// fn string_similarity(paths: &mut Vec<AnimePathWorking>, media_id: Option<i32>, anime_data: &HashMap<i32, AnimeInfo>) {

//     let mut previous_file_name = String::new();

//     // let mut folders = paths.first().unwrap().path.split("\\");
//     // let index = folders.clone().count();
//     // let folder = format!("data/{}_string_similarity.txt",folders.nth(index-2).unwrap());
//     // let path = Path::new(folder.as_str());

//     // if path.exists() {
//     //     match fs::remove_file(path) {
//     //         Err(why) => panic!("unable to remove, {}", why),
//     //         Ok(file) => file,
//     //     };
//     // }

//     // // create the file
//     // let mut file = match File::create(path) {
//     //     Err(why) => panic!("unable to open, {}", why),
//     //     Ok(file) => file,
//     // };

//     //let total = paths.len();
//     //let mut count = 0;

//     paths.iter_mut().for_each(|path| {
//         //count += 1;

//         // skip files that have the same title or have the same first 6 characters
//         let number_of_characters = 6;
//         if path.filename == previous_file_name || 
//         (previous_file_name.len() >= number_of_characters && 
//         path.filename.chars().take(number_of_characters).eq(previous_file_name.chars().take(number_of_characters))) {
//             return;
//         }
//         else {
//             previous_file_name = path.filename.clone();
//         }

//         let (id, _title, similarity_score) = identify_media_id(&path.filename, &anime_data, media_id);
        
//         if similarity_score > constants::SIMILARITY_SCORE_THRESHOLD && similarity_score > path.similarity_score {
//             path.media_id = id;
//             path.similarity_score = similarity_score;
//         }
//         //println!("{} | {} / {}", path.filename, count, total);
//     });

//     // fill in data for files that were skipped
//     for i in 1..paths.len() {
//         if paths[i].filename == paths[i - 1].filename {
//             paths[i].similarity_score = paths[i - 1].similarity_score;
//             paths[i].media_id = paths[i - 1].media_id;
//         }
//     }

// }



// // returns the media id and similarity score based on the title
// pub fn identify_media_id(filename: &String, anime_data: &HashMap<i32,AnimeInfo>, only_compare: Option<i32>) -> (i32, String, f64) {

//     let mut score = 0.0;
//     let mut media_id = 0;
//     let mut title = String::new();
//     let pre_dash = Regex::new(r"([^-]*).*").unwrap();

//     if only_compare.is_none() {

//         anime_data.iter().for_each(|data| {
            
//             title_compare(data.1, filename, &mut score, &mut media_id, &mut title, true);
//         });

//         if score < constants::SIMILARITY_SCORE_THRESHOLD {

//             anime_data.iter().for_each(|data| {
            
//                 title_compare(data.1, filename, &mut score, &mut media_id, &mut title, false);
//             });
//         }

//         if score < constants::SIMILARITY_SCORE_THRESHOLD {

//             let captures = pre_dash.captures(filename);
//             if captures.is_some() {

//                 let modified_filename = captures.unwrap().get(1).unwrap().as_str().to_string();
//                 anime_data.iter().for_each(|data| {
                    
//                     title_compare(data.1, &modified_filename, &mut score, &mut media_id, &mut title, false);
//                 });
//             }
//         }

//     } else if anime_data.contains_key(&only_compare.unwrap()) {
        
//         let anime = anime_data.get(&only_compare.unwrap()).unwrap();
//         title_compare(anime, filename, &mut score, &mut media_id, &mut title, true);

//         if score < constants::SIMILARITY_SCORE_THRESHOLD {
//             title_compare(anime, filename, &mut score, &mut media_id, &mut title, false);
//         }

//         if score < constants::SIMILARITY_SCORE_THRESHOLD {

//             if let Some(captures) = pre_dash.captures(filename) {

//                 let modified_filename = captures.get(1).unwrap().as_str().to_string();
                
//                 title_compare(anime, &modified_filename, &mut score, &mut media_id, &mut title, false);
//             }
//         }
//     }
//     (media_id, title, score)
// }



// // compare the title from the filename to all titles in the users anime list
// // character_skip will skip titles that don't have the same first letter as the filename title, this is a faster comparison but in rare cases may skip the matching anime title
// fn title_compare(anime: &AnimeInfo, filename: &String, score: &mut f64, media_id: &mut i32, return_title: &mut String, character_skip: bool) {
    
//     if filename.len() == 0 {
//         return;
//     }

//     let mut titles: Vec<String> = Vec::new();
//     if anime.title.english.is_some() { titles.push(replace_special_vowels(anime.title.english.clone().unwrap().to_ascii_lowercase())) }
//     if anime.title.romaji.is_some() { titles.push(replace_special_vowels(anime.title.romaji.clone().unwrap().to_ascii_lowercase())) }
//     if anime.title.custom.is_some() { titles.push(replace_special_vowels(anime.title.custom.clone().unwrap().to_ascii_lowercase())) }

//     for title in titles {

//         if character_skip == true && title.chars().next().unwrap() != filename.chars().next().unwrap() { 
//             continue;  // skip comparison if first character does not match
//         }
//         let no_special_vowels_filename = replace_special_vowels(filename.to_ascii_lowercase());
//         let normalized_levenshtein_score = strsim::normalized_levenshtein(&no_special_vowels_filename, &title);
//         if normalized_levenshtein_score > *score { 
//             *media_id = anime.id; 
//             *score = normalized_levenshtein_score;
//             *return_title = title.clone();
//         }
//     }
// }



// lazy_static! {
//     static ref REPLACE_A: Regex = Regex::new(r"À|Á|Â|Ã|Ä|Å|à|á|â|ã|ä|å").unwrap();
//     static ref REPLACE_AE: Regex = Regex::new(r"Æ|æ").unwrap();
//     static ref REPLACE_C: Regex = Regex::new(r"Ç|ç").unwrap();
//     static ref REPLACE_E: Regex = Regex::new(r"È|É|Ê|Ë|è|é|ê|ë").unwrap();
//     static ref REPLACE_I: Regex = Regex::new(r"Ì|Í|Î|Ï|ì|í|î|ï").unwrap();
//     static ref REPLACE_D: Regex = Regex::new(r"Ð|ð").unwrap();
//     static ref REPLACE_N: Regex = Regex::new(r"Ñ|ñ").unwrap();
//     static ref REPLACE_O: Regex = Regex::new(r"Ò|Ó|Ô|Õ|Ö|Ø|ò|ó|ô|õ|ö|ø").unwrap();
//     static ref REPLACE_U: Regex = Regex::new(r"Ù|Ú|Û|Ü|ù|ú|û|ü").unwrap();
//     static ref REPLACE_Y: Regex = Regex::new(r"Ý|ý|ÿ").unwrap();
//     static ref REPLACE_B: Regex = Regex::new(r"ß|Þ|þ").unwrap();
// }



// // replaces vowels with special marks
// // most people don't have these characters on their keyboard so they may create a discrepancy between the filename and official title
// fn replace_special_vowels(text: String) -> String {

//     let mut result = REPLACE_A.replace_all(&text, "a").to_string();
//     result = REPLACE_AE.replace_all(&result, "ae").to_string();
//     result = REPLACE_C.replace_all(&result, "c").to_string();
//     result = REPLACE_E.replace_all(&result, "e").to_string();
//     result = REPLACE_I.replace_all(&result, "i").to_string();
//     result = REPLACE_D.replace_all(&result, "d").to_string();
//     result = REPLACE_N.replace_all(&result, "n").to_string();
//     result = REPLACE_O.replace_all(&result, "o").to_string();
//     result = REPLACE_U.replace_all(&result, "u").to_string();
//     result = REPLACE_Y.replace_all(&result, "y").to_string();
//     result = REPLACE_B.replace_all(&result, "b").to_string();

//     result
// }



// // find the episode number in the filename and store it
// fn identify_episode_number(paths: &mut Vec<AnimePathWorking>) {

//     paths.iter_mut().for_each(|name| {

//         let episode = identify_number(&name.filename);
//         if episode.1 != 0 {
//             name.episode = episode.1;
//             name.filename = name.filename.replace(episode.0.as_str(), "");
//         }
//     });
// }



// // applies multiple regex to find the episode number
// // returns the text containing episode information, the episode, and the number of episodes in the file
// pub fn identify_number(filename: &String) -> (String, i32, i32) {

//     let captures = Regex::new(r"[^sS](\d+)[&-](\d+)").unwrap().captures(filename);
//     if captures.is_some() {
//         let captures2 = captures.unwrap();
//         let episode = captures2.get(1).unwrap().as_str().parse().unwrap();
//         let length = 1 + captures2.get(2).unwrap().as_str().parse::<i32>().unwrap() - episode;
//         return (captures2.get(0).unwrap().as_str().to_string(), episode, length);
//     }

//     // remove episode titles with numbers that would be misidentified as episode numbers
//     let episode_title_number = Regex::new(r"'.*\d+.*'").unwrap();
//     let filename_episode_title_removed = episode_title_number.replace_all(&filename, "").to_string();

//     // most anime fit this format
//     let num1 = extract_number(&filename_episode_title_removed, Regex::new(r" - (\d+)").unwrap());
//     if num1.1 != 0 {
//         return (num1.0, num1.1, 1);
//     }
//     // less common formats
//     let num2 = extract_number(&filename_episode_title_removed, Regex::new(r" - Episode (\d+)").unwrap());
//     if num2.1 != 0 {
//         return (num2.0, num2.1, 1);
//     }
//     let num3 = extract_number(&filename_episode_title_removed, Regex::new(r"[eE][pP]? ?(\d+)").unwrap());
//     if num3.1 != 0 {
//         return (num3.0, num3.1, 1);
//     }
//     // wider search for numbers, use last number that is not a version or season number
//     let num4 = extract_number(&filename_episode_title_removed, Regex::new(r"[^vsVS](\d+)").unwrap());
//     if num4.1 != 0 && num4.0 != "x264" {
//         return (num4.0, num4.1, 1);
//     }

//     (String::new(), 0, 0)
// }



// // finds and returns the episode number and wider string according to the regex rules
// fn extract_number(filename: &String, regex: Regex) -> (String, i32) {

//     let last_match = regex.find_iter(&filename).last();
//     // no number found
//     if last_match.is_none() { 
//         return (String::new(),0)
//     }

//     let episode = last_match.unwrap().as_str();
//     let captures = regex.captures(episode).unwrap();
//     (episode.to_string(), captures.get(1).unwrap().as_str().parse().unwrap())
// }


// // write episode number found into a file
// /*
// fn file_dump_episode(paths: &mut Vec<AnimePathWorking>) {
    
//     let path = Path::new("data/episode_data.txt");
//     let mut file: File;
//     // create the file
//     file = match File::create(path) {
//         Err(why) => panic!("unable to open, {}", why),
//         Ok(file) => file,
//     };

//     paths.iter().for_each(|name| { 
        
//         match write!(&mut file, "\n{} {}",name.filename, name.episode) {
//             Err(why) => panic!("ERROR: {}", why),
//             Ok(file) => file,
//         };
//     });
// }
// */


// fn irrelevant_information_removal_paths(paths: &mut Vec<AnimePathWorking>) {
    
//     paths.iter_mut().for_each(|name| {

//         name.filename = irrelevant_information_removal(name.filename.clone());
//     });
// }



// // regex used to filter out useless information
// lazy_static! {
//     static ref VERSION: Regex = Regex::new(r"[vV]\d+").unwrap();
//     static ref TRAILING_SPACES: Regex = Regex::new(r" +$").unwrap();
//     static ref TRAILING_DASH: Regex = Regex::new(r" - $").unwrap();
//     static ref TRAILING_DASH2: Regex = Regex::new(r" -$").unwrap();
//     static ref DOTS_AS_SPACES: Regex = Regex::new(r"\w\.\w").unwrap();
//     static ref EPISODE_TITLE: Regex = Regex::new(r"'.+'").unwrap();
//     static ref XVID: Regex = Regex::new(r"[xX][vV][iI][dD]").unwrap();
//     static ref SEASON_NUMBER: Regex = Regex::new(r" ?[sS]0(\d)").unwrap();
// }



// // remove any extra information that will interfere with comparing the filename with the anime title
// pub fn irrelevant_information_removal(filename: String) -> String {
    
//     // replace underscores with spaces to increase similarity with titles
//     let mut filename_clean = filename.replace("_", " ");

//     // replace dots with spaces to increase similarity with titles
//     if DOTS_AS_SPACES.is_match(&filename) {
//         filename_clean = filename_clean.replace(".", " ");
//     }

//     // remove extra information that is not part of the title
//     filename_clean = filename_clean.replace("dvd", "")
//         .replace("DVD", "")
//         .replace("Remastered", "")
//         .replace("remastered", "")
//         .replace(" Episode", "")
//         .replace(" Ep", "")
//         .replace(" EP", "")
//         .replace(" E ", "")
//         .replace(" END", "")
//         .replace(" FINAL", "")
//         .replace(" 1080p", "")
//         .replace(" 720p", "")
//         .replace(" 480p", "")
//         .replace(" BluRay", "")
//         .replace(" AV1", "")
//         .replace(" x264-ZQ", "")
//         .replace(" x264", "")
//         .replace(" DTS", "")
//         .replace(" AAC2", "")
//         .replace(" AAC2 0", "")
//         .replace(" WEB-DL", "")
//         .replace(" H264", "")
//         .replace(" H 264", "");
    
//     filename_clean = VERSION.replace_all(&filename_clean, "").to_string();
//     filename_clean = XVID.replace_all(&filename_clean, "").to_string();
//     filename_clean = TRAILING_DASH.replace_all(&filename_clean, "").to_string();
//     filename_clean = TRAILING_DASH2.replace_all(&filename_clean, "").to_string();
//     filename_clean = EPISODE_TITLE.replace_all(&filename_clean, "").to_string();
//     filename_clean = TRAILING_SPACES.replace_all(&filename_clean, "").to_string();
//     filename_clean = TRAILING_DASH2.replace_all(&filename_clean, "").to_string();
//     //filename_clean = SEASON_NUMBER.replace_all(&filename_clean, "").to_string();

//     let (episode_str, episode_i32) = extract_number(&filename_clean, SEASON_NUMBER.to_owned());
//     if episode_i32 > 1 {
//         let season_text = format!(" Season {}", episode_i32);
//         filename_clean = filename_clean.replace(&episode_str, &season_text);
//     } else {
//         filename_clean = filename_clean.replace(&episode_str, "");
//     }

//     // convert title to lowercase so the comparison doesn't think upper/lower case letters are different
//     filename_clean.to_ascii_lowercase()
// }



// pub fn extract_resolution(title: &String) -> i32 {

//     let resolution_1080 = Regex::new(r"1080p|1080|1920x1080").unwrap();
//     if resolution_1080.is_match(title) {
//         return 1080;
//     }

//     let resolution_720 = Regex::new(r"720p|720|960x720|1280x720").unwrap();
//     if resolution_720.is_match(title) {
//         return 720;
//     }

//     let resolution_480 = Regex::new(r"480p|480|720x480|852x480").unwrap();
//     if resolution_480.is_match(title) {
//         return 480;
//     }

//     let resolution_other = Regex::new(r"\d\d\d\d?x(\d\d\d\d?)").unwrap();
//     if resolution_other.is_match(title) {
//         let captures = resolution_other.captures(title).unwrap();
//         return captures.get(1).unwrap().as_str().parse().unwrap()
//     }

//     let resolution_other2 = Regex::new(r"(\d\d\d\d?)p").unwrap();
//     if resolution_other2.is_match(title) {
//         let captures = resolution_other2.captures(title).unwrap();
//         return captures.get(1).unwrap().as_str().parse().unwrap()
//     }

//     let dvd = Regex::new(r"([Dd][Vv][Dd])|[Ss][Dd]").unwrap();
//     if dvd.is_match(title) {
//         return 480;
//     }

//     0
// }



// pub fn extract_sub_group(title: &String) -> String {

//     let sub_group_find = Regex::new(r"^\[([^\[\]]+)\]").unwrap();
//     if sub_group_find.is_match(title) {
//         let captures = sub_group_find.captures(title).unwrap();
//         return captures.get(1).unwrap().as_str().to_string();
//     }

//     return String::new()
// }



// // replace a anime with its sequel if the episode number is too high
// fn replace_with_sequel_batch(paths: &mut Vec<AnimePathWorking>, anime_data: &HashMap<i32, AnimeInfo>) {

//     for path in paths {

//         replace_with_sequel(&mut path.media_id, &mut path.episode, &anime_data);
//     }
// }


// // replaces a anime with its sequel if its episode number is higher than the number of episodes
// // for example: episode 27 of a 26 episode series is episode 1 of season 2
// pub fn replace_with_sequel(anime_id: &mut i32, episode: &mut i32, anime_data: &HashMap<i32, AnimeInfo>) {

//     // anime is not in list or anime has unknown number of episodes which means it has no sequels
//     if anime_data.contains_key(&anime_id) == false || anime_data.get(&anime_id).unwrap().episodes.is_none() {
//         return;
//     }

//     // episode is within episode count
//     let mut episodes = anime_data.get(&anime_id).unwrap().episodes.unwrap();
//     if *episode <= episodes {
//         return;
//     }

//     // start from the first season
//     let mut prequel_exists = true;
//     while prequel_exists {

//         prequel_exists = false;
//         for edge in anime_data.get(&anime_id).unwrap().relations.edges.iter() {

//             if edge.relation_type == "PREQUEL" && anime_data.contains_key(&edge.node.id) && anime_data.get(&edge.node.id).unwrap().format.as_ref().unwrap() == "TV" {
//                 *anime_id = edge.node.id;
//                 episodes = anime_data.get(&anime_id).unwrap().episodes.unwrap();
//                 prequel_exists = true;
//             }
//         }
//     }

//     // traverse across sequels until episode is within episode count
//     let mut sequel_exists = true;
//     while *episode > episodes && sequel_exists {

//         sequel_exists = false;
//         for edge in anime_data.get(&anime_id).unwrap().relations.edges.iter() {
//             let mut format: String = String::from("");
//             if let Some(anime_data_entry) = anime_data.get(&edge.node.id) {
//                 if let Some(anime_format) = anime_data_entry.format.clone() {
//                     format = anime_format;
//                 }
//             }
//             if edge.relation_type == "SEQUEL" && format == "TV" {
//                 *anime_id = edge.node.id;
//                 *episode -= episodes;
//                 sequel_exists = true;
//                 break;
//             }
//         }
//         if anime_data.get(&anime_id).unwrap().episodes.is_none() {
//             break;
//         }
//         episodes = anime_data.get(&anime_id).unwrap().episodes.unwrap();
//     }
// }



// // get anime data for prequels of any anime that is in anime data global
// // necessary for recognizing anime that is labeled as one anime but belongs to a sequel of that anime
// // for example boku no hero academia episode 100 when no season has 100 episodes
// pub async fn get_prequel_data() {

//     let mut anime_data = GLOBAL_ANIME_DATA.lock().await;
//     let mut get_info: Vec<i32> = Vec::new();

//     for (_, anime) in anime_data.iter() {

//         for edge in anime.relations.edges.iter() {

//             if edge.relation_type == "PREQUEL" && edge.node.media_type == "ANIME" && anime_data.contains_key(&edge.node.id) == false {

//                 get_info.push(edge.node.id);
//                 println!("{} {}", anime.title.romaji.as_ref().unwrap(), edge.node.id);
//             }
//         }
//     }

//     while get_info.is_empty() == false {
//         println!("get_info size {}", get_info.len());
//         println!("{:?}", get_info);
//         match api_calls::anilist_api_call_multiple(get_info.clone()).await {
//             Ok(_result) => {
//                 GLOBAL_REFRESH_UI.lock().await.no_internet = false;
//                 let anime_ids = get_info.clone();
//                 get_info.clear();
//                 for id in anime_ids {
        
//                     if anime_data.contains_key(&id) == false {
        
//                         continue;
//                     }
//                     for edge in anime_data.get(&id).unwrap().relations.edges.iter() {
        
//                         if edge.relation_type == "PREQUEL" && anime_data.contains_key(&edge.node.id) == false {
            
//                             get_info.push(edge.node.id);
//                         }
//                     }
//                 }
//             },
//             Err(error) => {
//                 if error == "no connection" {
//                     GLOBAL_REFRESH_UI.lock().await.no_internet = true;
//                     break;
//                 } else {
//                     println!("error getting prequel data: {}", error);
//                 }
//             },
//         }
//     }
//     file_operations::write_file_anime_info_cache(&anime_data);
// }



// fn episode_fix_batch(paths: &mut Vec<AnimePathWorking>, anime_data: &HashMap<i32, AnimeInfo>) {

//     paths.iter_mut().for_each(|entry| {

//         episode_fix(entry.media_id, &mut entry.episode, &anime_data);
//     });
// }



// // will fix the episode number for numbers in titles of movies, ova's, etc
// pub fn episode_fix(anime_id: i32, episode: &mut i32, anime_data: &HashMap<i32, AnimeInfo>) {

//     let anime = anime_data.get(&anime_id);
//     if anime.is_some() {

//         let episodes = anime.unwrap().episodes;
//         if episodes.is_some() {

//             if episodes.unwrap() == 1 {

//                 *episode = 1;
//             }
//         }
//     }
// }