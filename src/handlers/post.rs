use crate::handlers::wiki::WikiPost;
use chrono::prelude::{DateTime, NaiveDate};
use glob::glob;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::cmp::Ordering;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::str;

static DATEFORMAT: &str = "M%m-%d-%Y";

#[derive(Clone, Serialize, Deserialize, Default, Eq)]
pub struct PostList {
    pub metadata: Metadata, // Metainformation
    pub content: String,    // Body
}

impl Ord for PostList {
    fn cmp(&self, other: &Self) -> Ordering {
        let date1 = NaiveDate::parse_from_str(&self.metadata.date, DATEFORMAT).unwrap();
        let date2 = NaiveDate::parse_from_str(&other.metadata.date, DATEFORMAT).unwrap();
        date1.cmp(&date2)
    }
}

impl PartialOrd for PostList {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PostList {
    fn eq(&self, other: &Self) -> bool {
        let date1 = NaiveDate::parse_from_str(&self.metadata.date, DATEFORMAT).unwrap();
        let date2 = NaiveDate::parse_from_str(&other.metadata.date, DATEFORMAT).unwrap();
        date1 == date2
    }
}

pub struct ContextState {
    pub repos: Vec<std::path::PathBuf>,
    pub posts: Vec<PostList>,
    pub wiki: Vec<WikiPost>,
    pub secret: String,
}

#[derive(Clone, Serialize, Deserialize, Eq, Debug, Default)]
pub struct Metadata {
    #[serde(default = "default_date")]
    pub date: String,
    #[serde(default = "default_author")]
    pub author: String,
    #[serde(default = "default_title")]
    pub title: String,
}

fn default_date() -> String {
    "M001-00-0000".to_string()
}

fn default_author() -> String {
    "i4".to_string()
}

fn default_title() -> String {
    "unknown".to_string()
}

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        let dateformat = "M%m-%d-%Y";
        let date1 = DateTime::parse_from_str(&self.date, dateformat).unwrap();
        let date2 = DateTime::parse_from_str(&other.date, dateformat).unwrap();
        date1 == date2
    }
}

fn post(path: PathBuf) -> Result<PostList, serde_yaml::Error> {
    println!("loading: {}", path.file_stem().unwrap().to_str().unwrap());
    let mut file_reader = std::fs::File::open(&path).expect("Could not open file");
    let mut content = String::new();

    let metadata: Metadata = match serde_yaml::Deserializer::from_reader(&file_reader)
        .take(1)
        .next()
    {
        Some(doc) => {
            serde_yaml::from_value(Value::deserialize(doc).expect("Could not deserialize metadata"))
                .unwrap_or_default()
        }
        None => Metadata::default(),
    };

    // Rewind file
    file_reader.seek(SeekFrom::Start(0)).unwrap();
    file_reader.read_to_string(&mut content).unwrap();

    // only interested in content
    // there is a better solution but meh
    content = content.split("---").skip(2).collect::<String>();

    Ok(PostList { metadata, content })
}

pub fn load(dir: &Path) -> Result<Vec<PostList>, String> {
    let mut posts: Vec<PostList> = glob(&format!("{}/*.md", dir.to_str().unwrap()))
        .expect("Failed to read pattern")
        .filter_map(Result::ok)
        .map(|fname| post(fname).unwrap_or_default())
        .collect();

    // sort
    posts.sort();
    posts.reverse();

    Ok(posts)
}
