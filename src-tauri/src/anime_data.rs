use std::{cmp::Ordering, collections::{HashMap, HashSet}};

use regex::Regex;
use serde::{Serialize, Deserialize};

use crate::{api_calls, user_data::UserInfo, constants};



#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AnimeInfo {
    pub average_score: Option<i32>,
    pub cover_image: CoverImage,
    pub description: Option<String>,
    pub duration: Option<i32>,
    pub episodes: Option<i32>,
    pub format: Option<String>,
    pub genres: Vec<String>,
    pub id: i32,
    pub is_adult: bool,
    pub popularity: i32,
    pub season: Option<String>,
    pub season_year: Option<i32>,
    pub start_date: AnilistDate,
    pub title: Title,
    pub trailer: Option<TrailerData>,
    pub media_type: String, // type is a rust keyword
    pub relations: Relations,
    pub recommendations: Option<Recommendations>,
    pub tags: Vec<Tag>,
    pub trending: i32,
    pub studios: Studio,
    pub next_airing_episode: Option<NextAiringEpisode>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CoverImage {
    pub large: String
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AnilistDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>
}

impl Ord for AnilistDate {
    fn cmp(&self, other: &Self) -> Ordering {

        if self.year.is_none() && other.year.is_some() {  return Ordering::Less }
        if self.year.is_some() && other.year.is_none() { return Ordering::Greater }
        if self.year.is_some() && other.year.is_some() {
            if self.year.unwrap() < other.year.unwrap() { return Ordering::Less }
            if self.year.unwrap() > other.year.unwrap()  { return Ordering::Greater }
        }

        if self.month.is_none() && other.month.is_some() { return Ordering::Less }
        if self.month.is_some() && self.month.is_none() { return Ordering::Greater } 
        if self.month.is_some() && other.month.is_some() {
            if self.month.unwrap() < other.month.unwrap() { return Ordering::Less } 
            if self.month.unwrap() > other.month.unwrap()  { return Ordering::Greater }
        }

        if self.day.is_none() && other.day.is_some() { return Ordering::Less } 
        if self.day.is_some() && self.day.is_none() { return Ordering::Greater } 
        if self.day.is_some() && other.day.is_some() {
            if self.day.unwrap() < other.day.unwrap() { return Ordering::Less } 
            if self.day.unwrap() > other.day.unwrap()  { return Ordering::Greater }
        }

        Ordering::Equal
    }
}

impl PartialOrd for AnilistDate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AnilistDate {
    fn eq(&self, other: &Self) -> bool {
        (self.year, &self.month, &self.day) == (other.year, &other.month, &other.day)
    }
}

impl Eq for AnilistDate { }

impl AnilistDate {
    pub const fn new() -> AnilistDate {
        AnilistDate { year: None, month: None, day: None }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Title {
    pub english: Option<String>,
    pub native: Option<String>,
    pub romaji: Option<String>,
    pub user_preferred: Option<String>,
    pub custom: Option<String>
}

// the structs below replicate the structure of data being returned by anilist api calls
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrailerData {
    pub id: String,
    pub site: String
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Relations {
    pub edges: Vec<Edge>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    pub relation_type: String,
    pub node: Node,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub id: i32,
    pub title: Title,
    pub cover_image: CoverImage,
    pub media_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Recommendations {
    pub nodes: Vec<RecNode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecNode {
    pub rating: i32,
    pub media_recommendation: Option<MediaRecommendation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct  MediaRecommendation {
    pub id: i32,
    pub title: Title,
    pub cover_image: CoverImage,
    pub media_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Tag {
    pub name: String,
    pub is_general_spoiler: bool,
    pub is_media_spoiler: bool,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Studio {
    pub nodes: Vec<NodeName>
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NodeName {
    pub name: String,
    pub is_animation_studio: bool
}


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NextAiringEpisode {
    pub airing_at: i32,
    pub episode: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct IdentifyInfo {
    pub filename: String,
    pub file_title: String,
    pub media_id: i32,
    pub similarity_score: f64,
    pub media_title: String,
    pub sub_group: String,
    pub resolution: i32,
    pub episode: i32,
    pub episode_length: i32,
}

impl IdentifyInfo {
    pub const fn new() -> IdentifyInfo {
        IdentifyInfo { filename: String::new(), file_title: String::new(), media_id: 0, similarity_score: 0.0, media_title: String::new(), sub_group: String::new(), resolution: 0, episode: 0, episode_length: 0 }
    }
}

// spell-checker:disable
lazy_static! {
    static ref VERSION: Regex = Regex::new(r"[vV][1-9][^9]").unwrap();
    static ref SUB_GROUP: Regex = Regex::new(r"^\[([^\]]+)\] ?").unwrap();
    static ref SUB_GROUP_BACKUP: Regex = Regex::new(r"-([^\-]+)$").unwrap();
    static ref RESOLUTION: Regex = Regex::new(r"(\d{3,4})[pP]").unwrap();
    static ref RESOLUTION2: Regex = Regex::new(r"\d{3,4} ?[xX] ?(\d{3,4})").unwrap();
    static ref EMPTY_BRACKETS: Regex = Regex::new(r"(\[[ ,\-]*\])|(\([ ,\-]*\))").unwrap();
    static ref CHECKSUM: Regex = Regex::new(r"\[[0-9A-Fa-f]{6,8}\]|\([0-9A-Fa-f]{6,8}\)").unwrap();
    static ref EXTENSION_CHECK: Regex = Regex::new(r"(.mkv)|(.mp4)|(.avi) ?$").unwrap();
    static ref TRAILING_EMPTY_SPACE: Regex = Regex::new(r"[ \-\.]+$").unwrap();
    static ref REMOVE_INFO: Vec<&'static str> = vec![" mkv", " mp4", " avi", ".mkv", ".mp4", ".avi", "HEVC-10bit", "HEVC-10", "HEVC", "x264-KRP", "x264-KQRM", "x264-VARYG", "H 264-VARYG", "x264", "x265", "10bits", 
    "10bit", "10Bit", "Hi10P", "Hi10", "Bluray", "BluRay", "YUV420P10LE", "AVC-YUV420P10", 
    "AVC", "FLAC2 0", "FLAC 2 0", "2xFLAC", "3xFLAC", "FLAC", "English Subbed", "English Sub", "English Dub", "EnglIsh Dub", "WEB-DL", "AV1", "DualA", "Dual audio", "Multiple Subtitle",
    "BDrip", "BDRip", "US BD", "UK BD-Remux", "UK BD-Remux", "BD-Rip", "BDRIP", "JP.BD", "WEBDL", "BD", "h264_qsv", "h264", "DTSx2", "DTS", "-DualPlease", "Dual", "Opus2 0", "Opus", "2xOPUS", "3xOPUS", "OPUS", "Multi-Subs", 
    "Multi-Sub", "Multi-Audio", "EAC3", "E-AC3", "AC3", "AAC5 0", "AAC2 0", "2xAAC", "xHE-AAC", "AAC", "Multi Subs", "Multi Sub", "WebRip", "WEB-RIP", "WEBRip",
    "WEB", "ENG", " Eng", "Eng ", "CR", "FUNi-DL", "FUNi", "FUNI", "HIDI", "HID", "English-Japanese Audio", "Japanese Audio", "Audio", 
    "Remux", "4k", "4K", "NF", "X264", "Flac", "Bdrip", "dual audio", "multisub", "MultiSubs", "Multi", "Web-Rip", 
    "Webrip", "Web", "8bit", "JP", "WeTV-Corrected", "TV Rip", "TV", "RAW", "E-AC-3", "DUAL AUDIO", "MULTI-SUB", "MULTI",
    "EAC-3", "H 264-ZigZag", "264-ZigZag", "REMUX", "LPCM 2 0", "LPCM", "PCM 2 0", "PCM", "TrueHD", "AC-3", "END", "DVD-Rip", "DSNP", "-ZeroBuild", "Blu-Ray", "AMZN", 
    "SRTx4", "DDP5 1", "DDP5", "HDR10", "HDR", "DVDRip", "DVD", "DV", "DDP 2 0", "DDP2 0", "DDP2", "264-KQRM", "JA+EN", "264-VARYG", "JAP", "10-bit", "(Weekly)", "(Uncensored)", 
    "10 bits", "10 bit", "WVH", "WV", "VVC", "H.264", "H 264", "x 264", "DEXA", "143 8561fps", "ADN", "8-bit", "DTS-HD MA", "5 1", "SEV", "Kira ", 
    "10-Bit", "Blu-ray", "Jpn", "H265", "H 265", "h 264", "-YUV444P10", "DUAL-AUDIO", "VHS", "UHD", "-iAHD", "HD", "[Japanese]", "(Japanese)", "Remaster", "-Tsundere-Raws", "B-Global",
    "-LYS1TH3A", "-Emmid", "HULU", "REPACK", "(Dub)", "-Rapta", "-Lazy", "-YURASUKA", "[}", "MKV", "Donghua", "-aKraa", " » Myanime live", "REMASTERED", "ENCODE", "-ZR-",
    "h265", "dvd", "EN ", "Jap", "BILI", "WIP", "MA", "ASS" ];
}

lazy_static! {
    static ref REPLACE_A: Regex = Regex::new(r"À|Á|Â|Ã|Ä|Å|à|á|â|ã|ä|å").unwrap();
    static ref REPLACE_AE: Regex = Regex::new(r"Æ|æ").unwrap();
    static ref REPLACE_B: Regex = Regex::new(r"ß|Þ|þ").unwrap();
    static ref REPLACE_C: Regex = Regex::new(r"Ç|ç").unwrap();
    static ref REPLACE_D: Regex = Regex::new(r"Ð|ð").unwrap();
    static ref REPLACE_E: Regex = Regex::new(r"È|É|Ê|Ë|è|é|ê|ë").unwrap();
    static ref REPLACE_I: Regex = Regex::new(r"Ì|Í|Î|Ï|ì|í|î|ï").unwrap();
    static ref REPLACE_N: Regex = Regex::new(r"Ñ|ñ").unwrap();
    static ref REPLACE_O: Regex = Regex::new(r"Ò|Ó|Ô|Õ|Ö|Ø|ò|ó|ô|õ|ö|ø").unwrap();
    static ref REPLACE_U: Regex = Regex::new(r"Ù|Ú|Û|Ü|ù|ú|û|ü").unwrap();
    static ref REPLACE_Y: Regex = Regex::new(r"Ý|ý|ÿ").unwrap();
}
// spell-checker:enable

pub struct AnimeData {
    pub data : HashMap<i32, AnimeInfo>,
    pub nonexistent_ids: HashSet<i32>,
    pub needs_scan: Vec<i32>,
}

impl AnimeData {

    pub fn clear(&mut self) {
        self.data.clear();
        self.nonexistent_ids.clear();
    }

    pub async fn get_anime_data(&mut self, media_id: i32) -> Result<AnimeInfo, &'static str> {

        if self.nonexistent_ids.contains(&media_id) {
            return Err("Anime does not exist");
        }

        if let Some(anime) = self.data.get(&media_id) {
            return Ok(anime.clone());
        } else {
            match api_calls::anilist_get_anime_info_single2(media_id).await {
                Ok(result) => {
                    self.data.insert(result.id, result.clone());
                    return Ok(result);
                },
                Err(error) => return Err(error),
            }
        }
    }

    pub async fn get_anime_list_data(&mut self, id_list: Vec<i32>) -> Result<Vec<AnimeInfo>, &'static str> {
        
        let mut valid_ids: Vec<i32> = id_list.iter().map(|id| *id).filter(|id| self.nonexistent_ids.contains(id) == false).collect();

        let missing_ids: Vec<i32> = valid_ids.iter().map(|id| *id).filter(|id| self.data.contains_key(id) == false).collect();

        if missing_ids.is_empty() == false {
            match api_calls::anilist_api_call_multiple2(missing_ids.clone()).await {
                Ok(result) => {
                    let missing_from_anilist_ids = self.find_missing_ids(&missing_ids, &result);
                    for anime in result {
                        self.data.insert(anime.id, anime);
                    }
                    for id in missing_from_anilist_ids {
                        valid_ids.remove(valid_ids.iter().position(|v_id| *v_id == id).unwrap());
                    }
                },
                Err(error) => return Err(error),
            }
        }

        let mut list_anime: Vec<AnimeInfo> = Vec::new();
        for id in valid_ids {
            if let Some(anime) = self.data.get(&id) {
                list_anime.push(anime.clone());
            } else {
                println!("anime is missing");
            }
        }

        Ok(list_anime)
    }

    // find anime missing from anilist and add them to the nonexistent list
    fn find_missing_ids(&mut self, ids: &Vec<i32>, data: &Vec<AnimeInfo>) -> Vec<i32> {

        let mut missing_ids: Vec<i32> = Vec::new();
        for id in ids {
            let mut found = false;
            for anime in data {
                if anime.id == *id {
                    found = true;
                    break;
                }
            }
            if found == false {
                self.nonexistent_ids.insert(*id);
                missing_ids.push(*id);
            }
        }

        missing_ids
    }

    pub fn sort_list(&self, anime_list: &mut Vec<(AnimeInfo,UserInfo)>, sorting: Option<String>) {
        if let Some(sort_category) = sorting {
            match sort_category.as_str() {
                "Alphabetical_native" => anime_list.sort_by(|first, second| {
                    if let Some(first_title) = first.0.title.native.clone() {
                        if let Some(second_title) = second.0.title.native.clone() {
                            first_title.to_lowercase().partial_cmp(&second_title.to_lowercase()).unwrap()
                        } else {
                            std::cmp::Ordering::Less
                        }
                    } else {
                        std::cmp::Ordering::Greater
                    }
                }),
                "Alphabetical_romaji" => anime_list.sort_by(|first, second| {
                    if let Some(first_title) = first.0.title.romaji.clone() {
                        if let Some(second_title) = second.0.title.romaji.clone() {
                            first_title.to_lowercase().partial_cmp(&second_title.to_lowercase()).unwrap()
                        } else {
                            std::cmp::Ordering::Less
                        }
                    } else {
                        std::cmp::Ordering::Greater
                    }
                }),
                "Alphabetical_english" => anime_list.sort_by(|first, second| {
                    if let Some(first_title) = first.0.title.english.clone() {
                        if let Some(second_title) = second.0.title.english.clone() {
                            first_title.to_lowercase().partial_cmp(&second_title.to_lowercase()).unwrap()
                        } else if let Some(second_title) = second.0.title.romaji.clone() {
                            first_title.to_lowercase().partial_cmp(&second_title.to_lowercase()).unwrap()
                        } else {
                            std::cmp::Ordering::Less
                        }
                    } else if let Some(first_title) = first.0.title.romaji.clone() {
                        if let Some(second_title) = second.0.title.english.clone() {
                            first_title.to_lowercase().partial_cmp(&second_title.to_lowercase()).unwrap()
                        } else if let Some(second_title) = second.0.title.romaji.clone() {
                            first_title.to_lowercase().partial_cmp(&second_title.to_lowercase()).unwrap()
                        } else {
                            std::cmp::Ordering::Less
                        }
                    } else {
                        std::cmp::Ordering::Greater
                    }
                }),
                "Score" => anime_list.sort_by(|first, second| {
                    first.0.average_score.partial_cmp(&second.0.average_score).unwrap()
                }),
                "MyScore" => anime_list.sort_by(|first, second| {
                    first.1.score.partial_cmp(&second.1.score).unwrap()
                }),
                "Date" => anime_list.sort_by(|first, second| {
                    first.0.start_date.partial_cmp(&second.0.start_date).unwrap()
                }),
                "Popularity" => anime_list.sort_by(|first, second| {
                    first.0.popularity.partial_cmp(&second.0.popularity).unwrap()
                }),
                "Trending" => anime_list.sort_by(|first, second| {
                    first.0.trending.partial_cmp(&second.0.trending).unwrap()
                }),
                "Started" => anime_list.sort_by(|first, second| {
                    first.1.started_at.partial_cmp(&second.1.started_at).unwrap()
                }),
                "Completed" => anime_list.sort_by(|first, second| {
                    first.1.completed_at.partial_cmp(&second.1.completed_at).unwrap()
                }),
                &_ => (),
            }
        } else {

        }
    }

    pub fn identify_anime(&self, filename: String, media_id: Option<i32>) {
        
        if EXTENSION_CHECK.is_match(&filename) == false {
            // return nothing
        }
        
        let mut info = IdentifyInfo::new();
        info.filename = filename;
        info.file_title = info.filename.clone();


        info.file_title = info.file_title.replace("_", " ");    
        if info.file_title.matches(".").count() > 5 {
            info.file_title = info.file_title.replace(".", " ");
        }

        info.file_title = CHECKSUM.replace_all(&info.file_title, "").to_string();
        if let Some(capture) = RESOLUTION.captures(&info.file_title) {
            if let Some(get) = capture.get(1) {
                if let Ok(resolution) = get.as_str().parse() {
                    info.resolution = resolution;
                    info.file_title = RESOLUTION.replace_all(&info.file_title, "").to_string();
                }
            }
        } else if let Some(capture) = RESOLUTION2.captures(&info.file_title) {
            if let Some(get) = capture.get(1) {
                if let Ok(resolution) = get.as_str().parse() {
                    info.resolution = resolution;
                    info.file_title = RESOLUTION2.replace_all(&info.file_title, "").to_string();
                }
            }
        }

        for remove in REMOVE_INFO.iter() {
            info.file_title = info.file_title.replace(remove, "");
        }

        info.file_title = VERSION.replace_all(&info.file_title, "").to_string();
        info.file_title = EMPTY_BRACKETS.replace_all(&info.file_title, "").to_string();

        let (ep_string, ep, length) = self.identify_number(&info.file_title);
        if ep > 0 {
            match info.file_title.find(&ep_string) {
                Some(index) => info.file_title = info.file_title[..index].to_string(),
                None => println!("string missing"),
            }
        }
        //info.file_title = info.file_title.replace(&ep_string, "");
        info.episode = ep;
        info.episode_length = length;

        if SUB_GROUP.is_match(&info.file_title) {
            info.sub_group = SUB_GROUP.captures(&info.file_title).unwrap().get(1).unwrap().as_str().to_string();
            info.file_title = SUB_GROUP.replace_all(&info.file_title, "").to_string();
        } else {
            if let Some(capture) = SUB_GROUP_BACKUP.captures(&info.file_title) {
                info.sub_group = capture.get(1).unwrap().as_str().to_string();
            }
            info.file_title = SUB_GROUP_BACKUP.replace_all(&info.file_title, "").to_string();
        }
        
        info.file_title = TRAILING_EMPTY_SPACE.replace_all(&info.file_title, "").to_string();

        let (id, title, similarity_score) = self.identify_media_id(&info.file_title, media_id);
        info.media_id = id;
        info.media_title = title;
        info.similarity_score = similarity_score;

        self.replace_with_sequel(&mut info);
        self.episode_fix(&mut info);
    }

    fn identify_number(&self, filename: &String) -> (String, i32, i32) {

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
        let num1 = self.extract_number(&filename_episode_title_removed, Regex::new(r" - (\d+)").unwrap());
        if num1.1 != 0 {
            return (num1.0, num1.1, 1);
        }
        // less common formats
        let num2 = self.extract_number(&filename_episode_title_removed, Regex::new(r" - Episode (\d+)").unwrap());
        if num2.1 != 0 {
            return (num2.0, num2.1, 1);
        }
        let num3 = self.extract_number(&filename_episode_title_removed, Regex::new(r"[sS]\d+[eE][pP]? ?(\d+)").unwrap());
        if num3.1 != 0 {
            return (num3.0, num3.1, 1);
        }
        let num4 = self.extract_number(&filename_episode_title_removed, Regex::new(r"[eE][pP]? ?(\d+)").unwrap());
        if num4.1 != 0 {
            return (num4.0, num4.1, 1);
        }
        // wider search for numbers, use last number that is not a version or season number
        let num5 = self.extract_number(&filename_episode_title_removed, Regex::new(r"[^vsVS](\d+)").unwrap());
        if num5.1 != 0 && num5.0 != "x264" {
            return (num5.0, num5.1, 1);
        }
    
        (String::new(), 0, 0)
    }
    
    // finds and returns the episode number and wider string according to the regex rules
    fn extract_number(&self, filename: &String, regex: Regex) -> (String, i32) {
    
        let last_match = regex.find_iter(&filename).last();
        // no number found
        if last_match.is_none() { 
            return (String::new(),0)
        }
    
        let episode = last_match.unwrap().as_str();
        let captures = regex.captures(episode).unwrap();
        (episode.to_string(), captures.get(1).unwrap().as_str().parse().unwrap())
    }

    pub fn identify_media_id(&self, filename: &String, only_compare: Option<i32>) -> (i32, String, f64) {

        let mut score = 0.0;
        let mut media_id = 0;
        let mut title = String::new();
        let pre_dash = Regex::new(r"([^-]*).*").unwrap();
    
        if only_compare.is_none() {
    
            self.data.iter().for_each(|data| {
                
                self.title_compare(data.1, filename, &mut score, &mut media_id, &mut title, true);
            });
    
            if score < constants::SIMILARITY_SCORE_THRESHOLD {
    
                self.data.iter().for_each(|data| {
                
                    self.title_compare(data.1, filename, &mut score, &mut media_id, &mut title, false);
                });
            }
    
            if score < constants::SIMILARITY_SCORE_THRESHOLD {
    
                let captures = pre_dash.captures(filename);
                if captures.is_some() {
    
                    let modified_filename = captures.unwrap().get(1).unwrap().as_str().to_string();
                    self.data.iter().for_each(|data| {
                        
                        self.title_compare(data.1, &modified_filename, &mut score, &mut media_id, &mut title, false);
                    });
                }
            }
    
        } else if self.data.contains_key(&only_compare.unwrap()) {
            
            let anime = self.data.get(&only_compare.unwrap()).unwrap();
            self.title_compare(anime, filename, &mut score, &mut media_id, &mut title, true);
    
            if score < constants::SIMILARITY_SCORE_THRESHOLD {
                self.title_compare(anime, filename, &mut score, &mut media_id, &mut title, false);
            }
    
            if score < constants::SIMILARITY_SCORE_THRESHOLD {
    
                if let Some(captures) = pre_dash.captures(filename) {
    
                    let modified_filename = captures.get(1).unwrap().as_str().to_string();
                    
                    self.title_compare(anime, &modified_filename, &mut score, &mut media_id, &mut title, false);
                }
            }
        }
        (media_id, title, score)
    }

    // compare the title from the filename to all titles in the users anime list
    // character_skip will skip titles that don't have the same first letter as the filename title, this is a faster comparison but in rare cases may skip the matching anime title
    fn title_compare(&self, anime: &AnimeInfo, filename: &String, score: &mut f64, media_id: &mut i32, return_title: &mut String, character_skip: bool) {
        
        if filename.len() == 0 {
            return;
        }
    
        let mut titles: Vec<String> = Vec::new();
        if anime.title.english.is_some() { titles.push(self.replace_special_vowels(anime.title.english.clone().unwrap().to_ascii_lowercase())) }
        if anime.title.romaji.is_some() { titles.push(self.replace_special_vowels(anime.title.romaji.clone().unwrap().to_ascii_lowercase())) }
        if anime.title.custom.is_some() { titles.push(self.replace_special_vowels(anime.title.custom.clone().unwrap().to_ascii_lowercase())) }
    
        for title in titles {
    
            if character_skip == true && title.chars().next().unwrap() != filename.chars().next().unwrap() { 
                continue;  // skip comparison if first character does not match
            }
            let no_special_vowels_filename = self.replace_special_vowels(filename.to_ascii_lowercase());
            let normalized_levenshtein_score = strsim::normalized_levenshtein(&no_special_vowels_filename, &title);
            if normalized_levenshtein_score > *score { 
                *media_id = anime.id; 
                *score = normalized_levenshtein_score;
                *return_title = title.clone();
            }
        }
    }
    
    // replaces vowels with special marks
    // most people don't have these characters on their keyboard so they may create a discrepancy between the filename and official title
    fn replace_special_vowels(&self, text: String) -> String {
    
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
        
    // replaces a anime with its sequel if its episode number is higher than the number of episodes
    // for example: episode 27 of a 26 episode series is episode 1 of season 2
    fn replace_with_sequel(&self, anime: &mut IdentifyInfo) {

        // anime is not in list or anime has unknown number of episodes which means it has no sequels
        if self.data.contains_key(&anime.media_id) == false || self.data.get(&anime.media_id).unwrap().episodes.is_none() {
            return;
        }

        // episode is within episode count
        let mut episodes = self.data.get(&anime.media_id).unwrap().episodes.unwrap();
        if anime.episode <= episodes {
            return;
        }

        // start from the first season
        let mut prequel_exists = true;
        while prequel_exists {

            prequel_exists = false;
            for edge in self.data.get(&anime.media_id).unwrap().relations.edges.iter() {

                if edge.relation_type == "PREQUEL" && self.data.contains_key(&edge.node.id) && self.data.get(&edge.node.id).unwrap().format.as_ref().unwrap() == "TV" {
                    anime.media_id = edge.node.id;
                    episodes = self.data.get(&anime.media_id).unwrap().episodes.unwrap();
                    prequel_exists = true;
                }
            }
        }

        // traverse across sequels until episode is within episode count
        let mut sequel_exists = true;
        while anime.episode > episodes && sequel_exists {

            sequel_exists = false;
            for edge in self.data.get(&anime.media_id).unwrap().relations.edges.iter() {
                let mut format: String = String::from("");
                if let Some(anime_data_entry) = self.data.get(&edge.node.id) {
                    if let Some(anime_format) = anime_data_entry.format.clone() {
                        format = anime_format;
                    }
                }
                if edge.relation_type == "SEQUEL" && format == "TV" {
                    anime.media_id = edge.node.id;
                    anime.episode -= episodes;
                    sequel_exists = true;
                    break;
                }
            }
            if self.data.get(&anime.media_id).unwrap().episodes.is_none() {
                break;
            }
            episodes = self.data.get(&anime.media_id).unwrap().episodes.unwrap();
        }
    }

    // will fix the episode number for numbers in titles of movies, ova's, etc
    fn episode_fix(&self, info: &mut IdentifyInfo) {
    
        let anime = self.data.get(&info.media_id);
        if anime.is_some() {
    
            let episodes = anime.unwrap().episodes;
            if episodes.is_some() {
    
                if episodes.unwrap() == 1 {
    
                    info.episode = 1;
                    info.episode_length = 1;
                }
            }
        }
    }

}