use crate::app::config::IndexPage;
use crate::handlers::post;
use askama_axum::Template;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
};
use std::sync::{Arc, Mutex};

#[derive(Template)]
#[template(path = "blog.html")]
pub struct BlogIndex {
    pub posts: Vec<post::PostList>,
    pub site: String,
    pub title: Option<String>,
}

#[derive(Template)]
#[template(path = "blogpost.html")]
pub struct BlogPost {
    pub content: String,
    pub metadata: post::Metadata,
    pub site: String,
    pub title: Option<String>,
}

pub async fn blog(
    Extension(posts): Extension<Arc<Mutex<post::ContextState>>>,
    Extension(site): Extension<String>, 
    Extension(index): Extension<IndexPage>,
) -> BlogIndex {
    let post_list = posts.lock().unwrap().posts.clone();
    BlogIndex { posts: post_list, site, title: index.title}
}

pub async fn blog_post(
    Path(title): Path<String>,
    Extension(posts): Extension<Arc<Mutex<post::ContextState>>>,
    Extension(site): Extension<String>,
    Extension(index): Extension<IndexPage>,
) -> Result<BlogPost, StatusCode> {
    // not the best way...
    // TODO change this to a hashmap
    let post_list = posts.lock().unwrap().posts.clone();
    match post_list
        .iter()
        .cloned()
        .find(|i| i.metadata.title == title)
    {
        Some(post_entry) => Ok(BlogPost {
            content: post_entry.content,
            metadata: post_entry.metadata,
            site,
            title: index.title,
        }),
        None => Err(StatusCode::NOT_FOUND),
    }
}
