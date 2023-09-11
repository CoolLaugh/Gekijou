use std::{cmp::Ordering, collections::{HashMap, HashSet}, path::Path, time::{SystemTime, UNIX_EPOCH, Instant}, io::ErrorKind};
use std::hash::{Hash, Hasher};

use regex::Regex;
use serde::{Serialize, Deserialize};
use walkdir::WalkDir;

use crate::{api_calls, user_data::UserInfo, constants, GLOBAL_REFRESH_UI, file_operations};



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
    pub synonyms: Vec<String>,
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
pub struct IdentifyInfo {
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
    static ref REMOVE_INFO: Vec<&'static str> = vec![" mkv", " mp4", " avi", ".mkv", ".mp4", ".avi", "HEVC-10bit", "HEVC-10", "HEVC10", "HEVC", "x264-KRP", "x264-KQRM", "x264-VARYG", "H 264-VARYG", "x264", "x265", "10bits", 
    "10bit", "10Bit", "Hi10P", "Hi10", "Bluray", "BluRay", "YUV420P10LE", "AVC-YUV420P10", 
    "AVC", "FLAC2.0", "FLAC2 0", "FLAC 2 0", "2xFLAC", "3xFLAC", "FLAC", "English Subbed", "English Sub", "English Dub", "EnglIsh Dub", "WEB-DL", "AV1", "DualA", "Dual audio", "Multiple Subtitle",
    "BDrip", "BDRip", "US BD", "UK BD-Remux", "UK BD-Remux", "BD-Rip", "BDRIP", "JP.BD", "WEBDL", "BD", "h264_qsv", "H264", "h264", "DTSx2", "DTS", "-DualPlease", "Dual", "Opus2 0", "Opus", "2xOPUS", "3xOPUS", "OPUS", "Multi-Subs", 
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

lazy_static! {
    static ref EXTRA_VIDEOS: Regex = Regex::new(r"[ _\.][oO][pP]\d*([vV]\d)?[ _\.]|[ _\.]NCOP\d*([vV]\d)?[ _\.]|[ _\.]NCED\d*([vV]\d)?[ _\.]|[ _\.][eE][dD]\d*([vV]\d)?[ _\.]|[ _\.][sS]kit[ _\.]|[eE]nding|[oO]pening|[ _][pP][vV][ _]|[bB][dD] [mM][eE][nN][uU]").unwrap();
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AnimePath {
    pub path: String,
    pub similarity_score: f64,
}

// used to tally up the number of times a anime has been recommended
#[derive(Debug, Clone, Default)]
struct RecommendTally {
    pub id: i32,
    pub rating: f32,
}

#[derive(Clone)]
pub struct AnimeData {
    pub data : HashMap<i32, AnimeInfo>,
    pub nonexistent_ids: HashSet<i32>,
    pub needs_scan: Vec<i32>,
    pub anime_path: HashMap<i32, HashMap<i32,AnimePath>>,
    pub known_files: HashSet<u64>,
    pub new_anime: bool,
}

impl AnimeData {

    pub fn new() -> AnimeData {
        AnimeData { data: HashMap::new(), nonexistent_ids: HashSet::new(), needs_scan: Vec::new(), anime_path: HashMap::new(), known_files: HashSet::new(), new_anime: false }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.nonexistent_ids.clear();
    }

    pub fn contains_key(&self, media_id: i32) -> bool {
        self.data.contains_key(&media_id)
    }

    pub async fn read_files(&mut self) {
        
        file_operations::read_file_anime_info_cache(&mut self.data).await;
        file_operations::read_file_anime_missing_ids(&mut self.nonexistent_ids).await;
        file_operations::read_file_episode_path(&mut self.anime_path).await;
        file_operations::read_file_known_files(&mut self.known_files).await;
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
                    self.new_anime = true;
                    file_operations::write_file_anime_info_cache(&self.data).await;
                    return Ok(result);
                },
                Err(error) => return Err(error),
            }
        }
    }

    pub async fn get_anime_list_data(&mut self, id_list: Vec<i32>) -> Result<Vec<AnimeInfo>, &'static str> {
        
        // filter 404 ids out
        let mut valid_ids: Vec<i32> = id_list.iter().map(|id| *id).filter(|id| self.nonexistent_ids.contains(id) == false).collect();
        
        // check for missing ids
        let missing_ids: Vec<i32> = valid_ids.iter().map(|id| *id).filter(|id| self.data.contains_key(id) == false).collect();
        if missing_ids.is_empty() == false {
            match api_calls::anilist_api_call_multiple(missing_ids.clone()).await {
                Ok(result) => {
                    let missing_from_anilist_ids = self.find_missing_ids(&missing_ids, &result).await;
                    for anime in result {
                        self.data.insert(anime.id, anime);
                        self.new_anime = true;
                    }
                    for id in missing_from_anilist_ids {
                        valid_ids.remove(valid_ids.iter().position(|v_id| *v_id == id).unwrap());
                    }
                    file_operations::write_file_anime_info_cache(&self.data).await;
                },
                Err(error) => return Err(error),
            }
        }

        // check if airing time needs updating
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time Error").as_secs() as i32;
        for id in valid_ids.iter() {
            let next_episode = self.data.get(&id).unwrap().next_airing_episode.clone();
            if let Some(next_episode) = next_episode {
                if next_episode.airing_at < current_time {
                    match self.update_anime_data(*id).await {
                        Ok(_) => {},
                        Err(error) => {
                            println!("update_anime_data error: {}", error);
                        },
                    }
                }
            }
        }

        // compile data of valid ids
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

    pub async fn update_anime_data(&mut self, media_id: i32) -> Result<AnimeInfo, &'static str> {

        match api_calls::anilist_api_call(media_id).await {
            Ok(result) => {
                self.data.insert(media_id, result.clone());
                return Ok(result);
            },
            Err(error) => {
                Err(error)
            },
        }

    }

    // find anime missing from anilist and add them to the nonexistent list
    async fn find_missing_ids(&mut self, ids: &Vec<i32>, data: &Vec<AnimeInfo>) -> Vec<i32> {

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
        file_operations::write_file_anime_missing_ids(&self.nonexistent_ids).await;
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

    pub fn identify_anime(&self, filename: String, media_id: Option<i32>) -> Option<IdentifyInfo> {
        
        if EXTENSION_CHECK.is_match(&filename) == false {
            return None;
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

        Some(info)
    }

    fn identify_number(&self, filename: &String) -> (String, i32, i32) {

        let captures = Regex::new(r"[^sS](\d+)[&-](\d+)").unwrap().captures(filename);
        if captures.is_some() {
            let captures2 = captures.unwrap();
            let episode = captures2.get(1).unwrap().as_str().parse().unwrap();
            let length = 1 + captures2.get(2).unwrap().as_str().parse::<i32>().unwrap() - episode;
            return (captures2.get(0).unwrap().as_str().to_string(), episode, length);
        }

        let episode_patterns = vec![Regex::new(r" - (\d+)").unwrap(), // most anime fit this format
                                                Regex::new(r" - Episode (\d+)").unwrap(), // less common formats
                                                Regex::new(r"[sS]\d+[eE][pP]? ?(\d+)").unwrap(),
                                                Regex::new(r"[eE][pP]? ?(\d+)").unwrap(),
                                                Regex::new(r"[^vsVS](\d+)").unwrap()]; // wider search for numbers, use last number that is not a version or season number

        for pattern in episode_patterns {
            let (episode_string, episode_number) = self.extract_number(&filename, pattern);
            if episode_number != 0 {
                return (episode_string, episode_number, 1);
            }
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
        for synonym in anime.synonyms.clone() {
            titles.push(self.replace_special_vowels(synonym.to_ascii_lowercase()));
        }
    
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

    pub async fn set_custom_filename(&mut self, media_id: i32, filename: String) -> Result<(), &'static str> {
        if self.data.contains_key(&media_id) == false {
            Err("Anime does not exist")
        } else {
            self.data.entry(media_id).and_modify(|anime| anime.title.custom = Some(filename));
            file_operations::write_file_anime_info_cache(&self.data).await;
            Ok(())
        }
    }

    pub fn get_custom_filename(&self, media_id: i32) -> Option<String> {
        match self.data.get(&media_id) {
            Some(anime) => anime.title.custom.clone(),
            None => return None,
        }
    }

    pub async fn scan_new_ids(&mut self, folders: Vec<String>) {

        for id in self.needs_scan.clone() {
            self.scan_folders(folders.clone(), false, Some(id)).await;
        }
        self.needs_scan.clear();
    }

    pub async fn scan_folders(&mut self, folders: Vec<String>, skip_files: bool, media_id: Option<i32>) -> bool {

        self.remove_missing_files();

        // any anime that needs to be scanned will be scanned for if the following conditions are met
        if skip_files == false && media_id.is_none() {
            // a second scan is redundant
            self.needs_scan.clear();
        }

        let mut file_found = false;
        let mut count = 0;
        let known_files = self.known_files.clone();
        self.known_files.clear(); // clear this so missing files are removed
        GLOBAL_REFRESH_UI.lock().await.scan_data.total_folders = folders.len() as i32;
        for folder in folders {
            count += 1;
            let mut refresh_ui = GLOBAL_REFRESH_UI.lock().await;
            refresh_ui.scan_data.current_folder = count;
            refresh_ui.scan_data.completed_chunks = 0;
            drop(refresh_ui);
            if self.scan_folder(folder, skip_files, media_id, &known_files).await {
                file_found = true;
            }
        }
        file_operations::write_file_episode_path(&self.anime_path).await;
        file_operations::write_file_known_files(&self.known_files).await;
        let mut refresh_ui = GLOBAL_REFRESH_UI.lock().await;
        refresh_ui.scan_data.total_folders = 0;
        refresh_ui.scan_data.current_folder = 0;
        file_found
    }

    pub async fn scan_folder(&mut self, folder: String, skip_files: bool, media_id: Option<i32>, known_files: &HashSet<u64>) -> bool {

        let mut file_found = false;
        let path = Path::new(&folder);
        if path.exists() == false {
            return false;
        }

        let mut count = 0;
        
        let iter = WalkDir::new(path).into_iter();
        GLOBAL_REFRESH_UI.lock().await.scan_data.total_chunks = iter.count() as i32;

        let valid_extensions = ["mkv", "mp4", "avi"];
        for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {

            if entry.file_type().is_file() == false {
                continue;
            }

            count += 1;
            if count % 100 == 0 {
                println!("{}", count);
                GLOBAL_REFRESH_UI.lock().await.scan_data.completed_chunks = count;
            }

            if let Some(ext) = entry.path().extension() {
                if valid_extensions.contains(&ext.to_str().unwrap_or_default()) == false {
                    continue;
                }


                let path = entry.path().to_path_buf().to_str().unwrap().to_string();
                let file_name = path.split('\\').last().unwrap().to_string();

                if EXTRA_VIDEOS.is_match(&file_name) {
                    continue; // skip openings, endings, etc
                }

                // hash file name for privacy and check if hash has already been seen before
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                path.hash(&mut hasher);
                let hash = hasher.finish();
                self.known_files.insert(hash);
                if skip_files == true {
                    if known_files.contains(&hash) {
                        continue;
                    }
                }
                
                if let Some(identity) = self.identify_anime(file_name, media_id) {

                    if identity.media_id != 0 && identity.similarity_score > constants::SIMILARITY_SCORE_THRESHOLD {

                        let media = self.anime_path.entry(identity.media_id).or_default();
                        if media.contains_key(&identity.episode) {
                            media.entry(identity.episode).and_modify(|anime_path| {
                                if identity.similarity_score > anime_path.similarity_score {
                                    anime_path.similarity_score = identity.similarity_score;
                                    anime_path.path = path;
                                }
                            });
                        } else {
                            media.insert(identity.episode, AnimePath { path: path, similarity_score: identity.similarity_score });
                        }
                        file_found = true;
                    }
                }
            }
        }
        println!("scan finished");
        file_found
    }

    pub fn remove_missing_files(&mut self) {

        for (_, anime) in self.anime_path.iter_mut() {

            anime.retain(|_, episode| { Path::new(&episode.path).exists() });
        }
        self.anime_path.retain(|_,anime| { anime.len() > 0 });
    }

    pub async fn get_prequel_data(&mut self) {

        let mut get_info: Vec<i32> = Vec::new();
        for (_, anime) in self.data.iter() {

            for edge in anime.relations.edges.iter() {

                if edge.relation_type == "PREQUEL" && edge.node.media_type == "ANIME" && self.data.contains_key(&edge.node.id) == false {
                    
                    get_info.push(edge.node.id);
                }
            }
        }

        while get_info.is_empty() == false {
            println!("get_info size {}", get_info.len());
            println!("{:?}", get_info);
            match api_calls::anilist_api_call_multiple(get_info.clone()).await {
                Ok(_result) => {
                    
                    let anime_ids = get_info.clone();
                    get_info.clear();
                    for id in anime_ids {
            
                        if self.data.contains_key(&id) == false {
            
                            continue;
                        }
                        for edge in self.data.get(&id).unwrap().relations.edges.iter() {
            
                            if edge.relation_type == "PREQUEL" && self.data.contains_key(&edge.node.id) == false {
                
                                get_info.push(edge.node.id);
                            }
                        }
                    }
                },
                Err(error) => {
                    if error == "no connection" {
                        GLOBAL_REFRESH_UI.lock().await.no_internet = true;
                        break;
                    } else {
                        println!("error getting prequel data: {}", error);
                    }
                },
            }
        }
    }

    pub async fn play_episode(&self, anime_id: i32, episode: i32) -> bool {

        let mut episode_opened = false;
        if self.anime_path.contains_key(&anime_id) {
            let media = self.anime_path.get(&anime_id).unwrap();
            if let Some(media_episode) = media.get(&episode) {

                println!("Opening: {} {}", media_episode.path, media_episode.similarity_score);
                
                let next_episode_path = Path::new(&media_episode.path);
                match open::that(next_episode_path) {
                    Err(error) => {
                        match error.kind() {
                            ErrorKind::NotFound => {
                                println!("Episode missing or moved");
                            },
                            _ => { println!("{:?}", error); },
                        }
                    },
                    Ok(_e) => { episode_opened = true },
                }
            }
        }

        episode_opened
    }

    pub fn get_existing_files_all_anime(&self) -> HashMap<i32, Vec<i32>> {

        let mut episodes_exist: HashMap<i32, Vec<i32>> = HashMap::new();

        for (anime_id, episodes) in self.anime_path.iter() {
    
            let mut episode_list: Vec<i32> = Vec::new();
    
            for (episode, _) in episodes {
    
                episode_list.push(*episode);
            }
    
            episodes_exist.insert(*anime_id, episode_list);
        }
        episodes_exist
    }

    pub fn get_existing_files(&self, anime_id: i32) -> Vec<i32> {

        let mut episodes_exist: Vec<i32> = Vec::new();
        if let Some(paths) = self.anime_path.get(&anime_id) {
            paths.keys().for_each(|key| {
                episodes_exist.push(*key);
            });
        }
        episodes_exist
    }

    pub async fn recommendations(&self, completed_scores: HashMap<i32, f32>, user_anime: HashSet<i32>, score_format: Option<String>, mode: String, genre_filter: String, year_min_filter: i32, year_max_filter: i32, format_filter: String) -> Vec<i32> {

        let mut recommend_tally = if mode == "user_recommended" {
            self.tally_recommendations(completed_scores, user_anime, score_format)
        } else {
            self.related_recommendations(completed_scores, user_anime, score_format, mode).await
        };
        
        // if any filter is used
        self.filter_anime(&mut recommend_tally, genre_filter,year_min_filter, year_max_filter, format_filter).await;

        // reduce number of shows so the ui isn't overloaded with a large number of shows
        recommend_tally.truncate(100);
        // only keep anime ids, no other information is needed
        let recommend_list: Vec<_> = recommend_tally.iter().map(|anime| anime.id).collect();
        recommend_list

    }

    fn tally_recommendations(&self, completed_scores: HashMap<i32, f32>, user_anime: HashSet<i32>, score_format: Option<String>) -> Vec<RecommendTally> {

        let mut recommend_total: HashMap<i32, f32> = HashMap::new();
        for (id, score) in completed_scores {

            if let Some(anime_entry) = self.data.get(&id) {

                if let Some(recommendations) = &anime_entry.recommendations {

                    for rec in &recommendations.nodes {

                        if let Some(recommendation) = &rec.media_recommendation {

                            let score_modifier = self.score_to_rating_modifier(score, &score_format);
                            // add the recommendation to the list or add the rating to the existing recommendation
                            recommend_total.entry(recommendation.id)
                                .and_modify(|r| { *r += (rec.rating as f32) * score_modifier })
                                .or_insert((rec.rating as f32) * score_modifier);
                        }
                    }
                }
            }
        }

        // remove anime already in the users lists
        recommend_total.retain(|id, _| { user_anime.contains(&id) == false });
        
        // move to a vector so the entries can be sorted
        let mut recommendations: Vec<RecommendTally> = Vec::new();
        for entry in recommend_total {
            recommendations.push(RecommendTally {
                id: entry.0,
                rating: entry.1
            });
        }

        
        // sort by most recommended
        recommendations.sort_by(| entry_a, entry_b | {
            entry_b.rating.partial_cmp(&entry_a.rating).unwrap()
        });

        // remove the least recommended shows but not too many in case it will be further filtered
        recommendations.truncate(1000);

        recommendations
    }

    /// convert the score to a rating modifier. 
    /// higher scores will produce a higher modifier so shows that a user liked will be worth more than a show the user disliked
    fn score_to_rating_modifier(&self, score: f32, score_format: &Option<String>) -> f32 {

        // show has no score
        if score == 0.0 {
            return 1.0;
        }

        if score_format.is_none() {
            return 1.0;
        }

        // convert to modifier
        match score_format.as_ref().unwrap().as_str() {
            "POINT_100" => score / 50.0,
            "POINT_10_DECIMAL" => score / 5.0,
            "POINT_10" => score / 5.0,
            "POINT_5" => score / 2.5,
            "POINT_3" => score - 1.0,
            _ => 1.0,
        }
    }

        
    // remove anime from anime_list if it does not match the filters
    async fn filter_anime(&self, anime_list: &mut Vec<RecommendTally>, genre_filter: String, year_min_filter: i32, year_max_filter: i32, format_filter: String) {

        if genre_filter.is_empty() == false || year_min_filter != 0 || year_max_filter != 0 || format_filter.is_empty() == false {

            // filter out any show which doesn't match the genre
            if genre_filter.is_empty() == false {

                anime_list.retain(|rec| { 
                    self.data.get(&rec.id).unwrap().genres.contains(&genre_filter)
                })
            }

            // filter out any show which is too old
            if year_min_filter != 0 {

                anime_list.retain(|rec| { 
                    self.data.get(&rec.id).unwrap().season_year.is_some() &&
                    self.data.get(&rec.id).unwrap().season_year.unwrap() >= year_min_filter
                })
            }

            // filter out any show which is too new
            if year_max_filter != 0 {

                anime_list.retain(|rec| { 
                    self.data.get(&rec.id).unwrap().season_year.is_some() &&
                    self.data.get(&rec.id).unwrap().season_year.unwrap() <= year_max_filter
                })
            }

            // filter out any show which doesn't match the format
            if format_filter.is_empty() == false {

                anime_list.retain(|rec| { 
                    self.data.get(&rec.id).unwrap().format.is_some() &&
                    self.data.get(&rec.id).unwrap().format.as_ref().unwrap().eq(&format_filter)
                })
            }
        }
    }

        
    // create a list of recommended anime based on relations to anime in the completed list
    async fn related_recommendations(&self, completed_scores: HashMap<i32, f32>, user_anime: HashSet<i32>, score_format: Option<String>, mode: String) -> Vec<RecommendTally> {

        let mut recommend_total: HashMap<i32, f32> = HashMap::new();
        for (id, score) in completed_scores {

            if let Some(anime_entry) = self.data.get(&id) {

                let score_modifier = self.score_to_rating_modifier(score, &score_format);

                for related in &anime_entry.relations.edges {

                    // filter by relation type
                    if related.relation_type != mode {
                        continue;
                    }

                    // add anime id and score modifier or replace score modifier if the new modifier is higher
                    if recommend_total.contains_key(&related.node.id) == false {

                        recommend_total.insert(related.node.id, score_modifier);
                    } else if score_modifier > *recommend_total.get(&related.node.id).unwrap() {
                        
                        recommend_total.entry(related.node.id).and_modify(|entry| *entry = score_modifier);
                    }
                }
            }
        }

        // remove anime already in the users lists
        recommend_total.retain(|id, _| { user_anime.contains(&id) == false });

        // some ids lead to 404 pages, these ids won't be in anime_data, remove them
        recommend_total.retain(|anime_id, _| { self.data.contains_key(anime_id) == true });
        
        // sum up the number of recommendations for the anime and apply score modifier
        let mut recommendations: Vec<RecommendTally> = Vec::new();
        for (anime_id, score_modifier) in recommend_total {
            
            if let Some(anime_entry) = self.data.get(&anime_id) {
        
                if let Some(anime_recommendations) = &anime_entry.recommendations {
        
                    let score: i32 = anime_recommendations.nodes.iter().map(|r| r.rating).sum();

                    recommendations.push(RecommendTally { id: anime_id, rating: score as f32 * score_modifier });
                }
            }
        };

        // sort by most recommended
        recommendations.sort_by(| entry_a, entry_b | {
            entry_b.rating.partial_cmp(&entry_a.rating).unwrap()
        });

        recommendations
    }

    pub fn get_anime_episodes(&self) -> HashMap<i32, Option<i32>> {

        let mut episodes: HashMap<i32, Option<i32>> = HashMap::new();
        for (id, info) in &self.data {

            episodes.insert(*id, info.episodes);
        }

        episodes
    }

    // show will be searched for the next time folders are scanned. 
    // this scan will not skip files that were previously checked, 
    // instead it will skip comparing those files against multiple anime
    pub fn add_id_for_scanning(&mut self, media_id: i32) {
        self.needs_scan.push(media_id);
    }

}