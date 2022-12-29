use std::io::Cursor;

use regex::Regex;
use reqwest;
use xml;

use crate::file_name_recognition;

#[derive(Debug, Clone, Default)]
pub struct RssEntry {
    pub title: String,
    pub link: String,
    pub guid: String,
    pub pub_date: String,
    pub downloads: i32,
    pub info_hash: String,
    pub category_id: String,
    pub size: String,

    pub derived_values: DerivedValues,
}

#[derive(Debug, Clone, Default)]
pub struct DerivedValues {
    pub episode: i32,
    pub resolution: i32,
    pub sub_group: String,
}

pub async fn get_rss(search: String) {

    let url = format!("https://nyaa.si/?page=rss&q={}&c=1_2&f=0", search.replace(" ", "+"));

    let response = reqwest::get(url).await.unwrap().text().await.unwrap()
        .replace("\n", "")
        .replace("\t", "");

    let cursor = Cursor::new(response);

    // Parse the XML document
    let doc = xml::reader::EventReader::new(cursor);

    // Iterate through the events in the XML document
    let mut entry: RssEntry = RssEntry::default();
    let mut entrys: Vec<RssEntry> = Vec::new();

    let mut element_name = String::new();
    for event in doc {
        match event {
            Ok(xml::reader::XmlEvent::StartElement { name, attributes: _, .. }) => {
                element_name = name.local_name;
            }
            Ok(xml::reader::XmlEvent::Characters(text)) => {
                match element_name.as_str() {
                    "title" => { entry.title = text; },
                    "link" => { entry.link = text; },
                    "guid" => { entry.guid = text; },
                    "pubDate" => { entry.pub_date = text; },
                    "downloads" => { entry.downloads = text.parse().unwrap(); },
                    "infoHash" => { entry.info_hash = text; },
                    "categoryId" => { entry.category_id = text; },
                    "size" => { entry.size = text; },
                    &_ => (),
                }
            }
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

    for mut e in entrys {

        let mut title = e.title.clone();
        
        let valid_file_extensions = Regex::new(r"[_ ]?(\.mkv|\.avi|\.mp4)").unwrap();
        title = valid_file_extensions.replace_all(&title, "").to_string();

        e.derived_values.resolution = file_name_recognition::extract_resolution(&title);

        e.derived_values.sub_group = file_name_recognition::extract_sub_group(&title);

        title = file_name_recognition::remove_brackets(&title);

        e.derived_values.episode = file_name_recognition::identify_number(&title).1;

        println!("{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}", e.title, e.derived_values.sub_group, e.derived_values.resolution, e.derived_values.episode, title, e.link, e.guid, e.pub_date, e.downloads, e.info_hash, e.category_id, e.size);
    }
}