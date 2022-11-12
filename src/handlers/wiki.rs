use crate::handlers::{
    post::{ContextState, Metadata},
    status,
};
use askama_axum::Template;
use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum_macros::debug_handler;
use glob::glob;
use serde::Deserialize;
use serde_yaml::Value;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path as pathPath;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default, Clone)]
pub struct WikiPost {
    location: String,
    #[allow(dead_code)]
    // TODO: Will be used later to include author + date
    metadata: Metadata,
    content: String,
}

#[derive(Template, Debug, Default, Clone)]
#[template(path = "wiki.html")]
pub struct WikiIndex {
    content: Option<WikiPost>,
    documents: Vec<String>,
    current: String,
    folders: Vec<String>,
}

#[debug_handler]
pub async fn wiki_posts(
    Path(path): Path<PathBuf>,
    Extension(posts): Extension<Arc<Mutex<ContextState>>>,
) -> Result<WikiIndex, status::ErrorHandler> {
    let wiki_posts = posts.lock().unwrap().wiki.clone();
    let current = path.to_string_lossy().to_string().replacen('/', "", 1);

    let filtered: Vec<WikiPost> = wiki_posts
        .iter()
        .filter(|a| pathPath::new(&a.location).starts_with(&current))
        .cloned()
        .collect();

    // can be done much better
    let mut children: Vec<String> = filtered
        .iter()
        .map(|post| {
            post.location
                .replacen(&current, "", 1)
                .trim_start_matches('/')
                .to_string()
        })
        .map(|post| post.split('/').next().unwrap().to_string())
        .filter(|post| !post.is_empty())
        .collect();

    children.sort();
    children.dedup();

    // only get directories
    let folders: Vec<String> = children
        .iter()
        .filter(|post| !post.ends_with(".md"))
        .cloned()
        .collect();

    // only get directories
    let documents: Vec<String> = children
        .iter()
        .filter(|post| post.ends_with(".md"))
        .cloned()
        .collect();

    let content: Option<WikiPost> = filtered
        .iter()
        .filter(|post| post.location == current)
        .cloned()
        .collect::<Vec<WikiPost>>()
        .pop();

    if content.is_none() && content.is_none() && children.is_empty() {
        return Err(status::ErrorHandler {
            code: StatusCode::NOT_FOUND,
        });
    }

    Ok(WikiIndex {
        content,
        documents,
        current,
        folders,
    })
}

fn post(path: String, dir: String) -> Result<WikiPost, serde_yaml::Error> {
    let new_path = pathPath::new(&path);
    println!(
        "loading wiki: {}",
        new_path.file_stem().unwrap().to_str().unwrap()
    );
    let mut file_reader = std::fs::File::open(&new_path).expect("Could not open file");
    let mut content = String::new();

    let metadata: Metadata = match serde_yaml::Deserializer::from_reader(&file_reader)
        .into_iter()
        .take(1)
        .next()
    {
        Some(document) => match Value::deserialize(document) {
            Ok(v) => serde_yaml::from_value(v).unwrap_or_default(),
            Err(_) => Metadata::default(),
        },
        None => Metadata::default(),
    };

    // Rewind file
    file_reader.seek(SeekFrom::Start(0)).unwrap();
    file_reader.read_to_string(&mut content).unwrap();

    // only interested in content
    // there is a better solution but meh
    content = content.split("---").skip(2).collect::<String>();

    Ok(WikiPost {
        location: new_path
            .strip_prefix(dir)
            .unwrap()
            .to_string_lossy()
            .to_string(),
        metadata,
        content,
    })
}

pub fn load(dir: &pathPath) -> Option<Vec<WikiPost>> {
    println!("Reading wiki from {:#?}", &dir);
    let newdir = dir.to_string_lossy().to_string();
    let posts: Vec<WikiPost> = glob(&format!("{}/**/*.md", dir.to_str().unwrap()))
        .expect("Failed to read pattern")
        .filter_map(Result::ok)
        .map(|name| post(name.to_string_lossy().to_string(), newdir.clone()).unwrap_or_default())
        .collect();

    Some(posts)
}
