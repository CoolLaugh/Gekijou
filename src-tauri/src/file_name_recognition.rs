use std::fmt::format;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use regex::Regex;

use crate::GLOBAL_ANIME_DATA;
use similar_string::*;
use strsim;

#[derive(Debug, Clone, Default)]
struct AnimePath {
    path: String,
    filename: String,
    episode: i32,
    media_id: i32,
}



pub async fn parse_file_names(folders: Vec<String>) {

    for folder in folders {

        let path = Path::new(&folder);
        if path.exists() == false {
            continue;
        }

        let mut sub_folders: Vec<String> = Vec::new();
        sub_folders.push(folder.clone());
        let mut file_names: Vec<AnimePath> = Vec::new();

        while sub_folders.is_empty() == false {
            
            let sub_folder = sub_folders.remove(0);
            
            for file in fs::read_dir(sub_folder).unwrap() {

                let unwrap = file.unwrap();
                if unwrap.file_type().unwrap().is_dir() {
                    sub_folders.push(unwrap.path().to_str().unwrap().to_string());
                } else {
                    file_names.push(AnimePath {path: unwrap.path().to_str().unwrap().to_string(), filename: unwrap.file_name().to_str().unwrap().to_string(), episode: 0, media_id: 0});
                }
            }
        }

        // remove files that are not video files
        let valid_file_extentions = Regex::new(r"[_ ]?(\.mkv|\.avi|\.mp4)").unwrap();
        // remove openings, endings, PV, and other non episode videos
        let extra_videos = Regex::new(r"[ _\.]OP\d*([vV]\d)?[ _\.]|[ _\.]NCOP\d*([vV]\d)?[ _\.]|[ _\.]NCED\d*([vV]\d)?[ _\.]|[ _\.]ED\d*([vV]\d)?[ _\.]|[ _\.][sS]kit[ _\.]|[eE]nding|[oO]pening|[ _][pP][vV][ _]").unwrap();
        let mut filtered_file_names = file_names
            .into_iter()
            .filter(|name| valid_file_extentions.is_match(&name.filename) &&
                extra_videos.is_match(&name.filename) == false)
            .collect::<Vec<_>>();
        
            filtered_file_names.iter_mut().for_each(|name| {
            name.filename = valid_file_extentions.replace_all(&name.filename, "").to_string();
        });
    
        // remove brackets and their contents, the name and episode are unlikely to be here
        filtered_file_names.iter_mut().for_each(|name| {
            name.filename = Regex::new(r"((\[[^\[\]]+\]|\([^\(\)]+\))[ _]*)+").unwrap().replace_all(&name.filename, "").to_string();
        });

        // most anime fit this format
        let first_episode_filter = Regex::new(r" - (\d+)").unwrap();
        filtered_file_names.iter_mut().for_each(|name| {
            
            let last_capture = first_episode_filter.captures_iter(&name.filename).last();
            if last_capture.is_none() {
                return;
            }
            let captures = last_capture.unwrap();
            name.episode = captures.get(1).unwrap().as_str().parse().unwrap();
            name.filename = name.filename.replace(captures.get(0).unwrap().as_str(), "");
            //println!("{},{},{},{}", name.filename, captures.get(0).unwrap().as_str(), captures.get(1).unwrap().as_str(),name.episode);
        });

        // wider search for numbers, use last number that is not a version or season number
        let episode_filter = Regex::new(r"[^vsVS](\d+)").unwrap();
        filtered_file_names.iter_mut().for_each(|name| {

            if name.episode != 0 {
                return;
            }

            let numbers = episode_filter.captures(&name.filename);
            if numbers.is_none() { 
                return;
            }

            let numbers_unwrapped = numbers.unwrap();
            if numbers_unwrapped.len() == 0 {
                return;
            }

            let episode = numbers_unwrapped.get(numbers_unwrapped.len()-1).unwrap().as_str();
            name.episode = episode.parse().unwrap();
            name.filename = name.filename.replace(episode, "");
        });

        let version = Regex::new(r"[vV]\d+").unwrap();
        let trailing_spaces = Regex::new(r" +$").unwrap();
        let trailing_dash = Regex::new(r" - $").unwrap();
        let trailing_dash2 = Regex::new(r" -$").unwrap();
        let dots_as_spaces = Regex::new(r"\w\.\w").unwrap();
        let episode_title = Regex::new(r"'.+'").unwrap();
        let xvid = Regex::new(r"[xX][vV][iI][dD]]").unwrap();
        filtered_file_names.iter_mut().for_each(|name| {

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
            name.filename = name.filename.replace(" Ep", "");
            name.filename = name.filename.replace(" EP", "");
            name.filename = name.filename.replace(" E ", "");
            name.filename = name.filename.replace(" END", "");
            name.filename = name.filename.replace(" FINAL", "");

            name.filename = version.replace(&name.filename, "").to_string();
            name.filename = xvid.replace(&name.filename, "").to_string();
            name.filename = trailing_dash.replace(&name.filename, "").to_string();
            name.filename = trailing_dash2.replace(&name.filename, "").to_string();
            name.filename = episode_title.replace(&name.filename, "").to_string();
            name.filename = trailing_spaces.replace(&name.filename, "").to_string();
            name.filename = trailing_dash2.replace(&name.filename, "").to_string();
        });

        

        let anime_data = GLOBAL_ANIME_DATA.lock().await;

        let path = Path::new("match_data.txt");
        let mut file: File;
        // create the file
        file = match File::create(path) {
            Err(why) => panic!("unable to open, {}", why),
            Ok(file) => file,
        };

        let mut previous_file_name = String::new();
        let mut counter = 0;
        let total = filtered_file_names.len();
       
        match write!(&mut file, "--------------------------------------------------------------------------------------------------------------------------------") {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };

        filtered_file_names.iter_mut().for_each(|name| {

            counter += 1;
            println!("{}/{}",counter, total);
            if name.filename == previous_file_name {
                return;
            }
            else {
                previous_file_name = name.filename.clone();
            }

            let mut scores: Vec<f64> = Vec::new();

            {
                let mut highest_similarity = 0.0;
                let mut highest_similarity_media_name = String::new();

                anime_data.iter().for_each(|data| {

                    if data.1.title.english.is_some() {

                        let title = data.1.title.english.clone().unwrap();
                        let score = strsim::normalized_levenshtein(&name.filename, &title);
                        if score > highest_similarity { highest_similarity_media_name = title; highest_similarity = score; }
                    }
                    if data.1.title.romaji.is_some() {

                        let title = data.1.title.romaji.clone().unwrap();
                        let score = strsim::normalized_levenshtein(&name.filename, &title);
                        if score > highest_similarity { highest_similarity_media_name = title; highest_similarity = score; }
                    }
                    if data.1.title.native.is_some() {

                        let title = data.1.title.native.clone().unwrap();
                        let score = strsim::normalized_levenshtein(&name.filename, &title);
                        if score > highest_similarity { highest_similarity_media_name = title; highest_similarity = score; }
                    }
                });
                    
                match write!(&mut file, "\nnormalized_levenshtein:\t\t\t | {:.4} | {} | {}", highest_similarity, name.filename, highest_similarity_media_name) {
                    Err(why) => panic!("ERROR: {}", why),
                    Ok(file) => file,
                };
                scores.push(highest_similarity);
            }
            {
                let mut highest_similarity = 0.0;
                let mut highest_similarity_media_name = String::new();

                anime_data.iter().for_each(|data| {

                    if data.1.title.english.is_some() {

                        let title = data.1.title.english.clone().unwrap();
                        let score = strsim::normalized_damerau_levenshtein(&name.filename, &title);
                        if score > highest_similarity { highest_similarity_media_name = title; highest_similarity = score; }
                    }
                    if data.1.title.romaji.is_some() {

                        let title = data.1.title.romaji.clone().unwrap();
                        let score = strsim::normalized_damerau_levenshtein(&name.filename, &title);
                        if score > highest_similarity { highest_similarity_media_name = title; highest_similarity = score; }
                    }
                    if data.1.title.native.is_some() {

                        let title = data.1.title.native.clone().unwrap();
                        let score = strsim::normalized_damerau_levenshtein(&name.filename, &title);
                        if score > highest_similarity { highest_similarity_media_name = title; highest_similarity = score; }
                    }
                });
                    
                match write!(&mut file, "\nnormalized_damerau_levenshtein:  | {:.4} | {} | {}", highest_similarity, name.filename, highest_similarity_media_name) {
                    Err(why) => panic!("ERROR: {}", why),
                    Ok(file) => file,
                };
                scores.push(highest_similarity);
            }
            {
                let mut highest_similarity = 0.0;
                let mut highest_similarity_media_name = String::new();

                anime_data.iter().for_each(|data| {

                    if data.1.title.english.is_some() {

                        let title = data.1.title.english.clone().unwrap();
                        let score = strsim::jaro(&name.filename, &title);
                        if score > highest_similarity { highest_similarity_media_name = title; highest_similarity = score; }
                    }
                    if data.1.title.romaji.is_some() {

                        let title = data.1.title.romaji.clone().unwrap();
                        let score = strsim::jaro(&name.filename, &title);
                        if score > highest_similarity { highest_similarity_media_name = title; highest_similarity = score; }
                    }
                    if data.1.title.native.is_some() {

                        let title = data.1.title.native.clone().unwrap();
                        let score = strsim::jaro(&name.filename, &title);
                        if score > highest_similarity { highest_similarity_media_name = title; highest_similarity = score; }
                    }
                });
                    
                match write!(&mut file, "\njaro:\t\t\t\t\t\t\t | {:.4} | {} | {}", highest_similarity, name.filename, highest_similarity_media_name) {
                    Err(why) => panic!("ERROR: {}", why),
                    Ok(file) => file,
                };
                scores.push(highest_similarity);
            }
            {
                let mut highest_similarity = 0.0;
                let mut highest_similarity_media_name = String::new();

                anime_data.iter().for_each(|data| {

                    if data.1.title.english.is_some() {

                        let title = data.1.title.english.clone().unwrap();
                        let score = strsim::jaro_winkler(&name.filename, &title);
                        if score > highest_similarity { highest_similarity_media_name = title; highest_similarity = score; }
                    }
                    if data.1.title.romaji.is_some() {

                        let title = data.1.title.romaji.clone().unwrap();
                        let score = strsim::jaro_winkler(&name.filename, &title);
                        if score > highest_similarity { highest_similarity_media_name = title; highest_similarity = score; }
                    }
                    if data.1.title.native.is_some() {

                        let title = data.1.title.native.clone().unwrap();
                        let score = strsim::jaro_winkler(&name.filename, &title);
                        if score > highest_similarity { highest_similarity_media_name = title; highest_similarity = score; }
                    }
                });
                    
                match write!(&mut file, "\njaro_winkler:\t\t\t\t\t | {:.4} | {} | {}", highest_similarity, name.filename, highest_similarity_media_name) {
                    Err(why) => panic!("ERROR: {}", why),
                    Ok(file) => file,
                };
                scores.push(highest_similarity);
            }
            {
                let mut highest_similarity = 0.0;
                let mut highest_similarity_media_name = String::new();

                anime_data.iter().for_each(|data| {

                    if data.1.title.english.is_some() {

                        let title = data.1.title.english.clone().unwrap();
                        let score = strsim::sorensen_dice(&name.filename, &title);
                        if score > highest_similarity { highest_similarity_media_name = title; highest_similarity = score; }
                    }
                    if data.1.title.romaji.is_some() {

                        let title = data.1.title.romaji.clone().unwrap();
                        let score = strsim::sorensen_dice(&name.filename, &title);
                        if score > highest_similarity { highest_similarity_media_name = title; highest_similarity = score; }
                    }
                    if data.1.title.native.is_some() {

                        let title = data.1.title.native.clone().unwrap();
                        let score = strsim::sorensen_dice(&name.filename, &title);
                        if score > highest_similarity { highest_similarity_media_name = title; highest_similarity = score; }
                    }
                });
                    
                match write!(&mut file, "\nsorensen_dice:\t\t\t\t\t | {:.4} | {} | {}", highest_similarity, name.filename, highest_similarity_media_name) {
                    Err(why) => panic!("ERROR: {}", why),
                    Ok(file) => file,
                };
                scores.push(highest_similarity);
                    
                match write!(&mut file, "\nAverage:\t\t\t\t\t\t | {:.4} |", (scores[0] + scores[1] + scores[2] + scores[3] + scores[4])/5.0) {
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
            }
                    
            match write!(&mut file, "\n--------------------------------------------------------------------------------------------------------------------------------") {
                Err(why) => panic!("ERROR: {}", why),
                Ok(file) => file,
            };
        });

        println!("done");

        /* filtered_file_names.iter_mut().for_each(|name| {

            let mut highest_similarity = 0.0;
            let mut highest_similarity_media_id = 0;
            let mut highest_similarity_media_name = String::new();
            anime_data.iter().for_each(|data| {

                if data.1.title.english.is_some() {
                    
                    let similarity = compare_similarity(name.filename.clone(),data.1.title.english.clone().unwrap());
                    if similarity > highest_similarity {
                        highest_similarity = similarity;
                        highest_similarity_media_id = *data.0;
                        highest_similarity_media_name = data.1.title.english.clone().unwrap();
                    }
                }

                if data.1.title.romaji.is_some() {

                    let similarity = compare_similarity(name.filename.clone(),data.1.title.romaji.clone().unwrap());
                    if similarity > highest_similarity {
                        highest_similarity = similarity;
                        highest_similarity_media_id = *data.0;
                        highest_similarity_media_name = data.1.title.romaji.clone().unwrap();
                    }
                }

                if data.1.title.native.is_some() {

                    let similarity = compare_similarity(name.filename.clone(),data.1.title.native.clone().unwrap());
                    if similarity > highest_similarity {
                        highest_similarity = similarity;
                        highest_similarity_media_id = *data.0;
                        highest_similarity_media_name = data.1.title.native.clone().unwrap();
                    }
                }
            });

            println!("{} {} {}", name.filename, highest_similarity, highest_similarity_media_name);
        }); */
    }
}