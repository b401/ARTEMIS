use askama_axum::Template;
use axum::handler::HandlerWithoutStateExt;
use axum_macros::debug_handler;
use std::process::exit;
mod app {
    pub mod config;
    pub mod git;
}
mod handlers {
    pub mod blog;
    pub mod post;
    pub mod security;
    pub mod status;
    pub mod update;
    pub mod wiki;
}
use axum::{
    extract::Extension, http::StatusCode, routing::{post,get,get_service}, Router
};
use std::sync::{Arc, Mutex};
use tower_http::services::ServeDir;

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    site: String,
    slogan: Option<String>,
    title: Option<String>,
    skills: Option<Vec<String>>,
    github: Option<String>,
    mail: Option<String>,
    matrix: Option<String>,
    threema: Option<String>,
}

#[debug_handler]
async fn index(Extension(site): Extension<String>, Extension(contact): Extension<app::config::Contact>, Extension(index): Extension<app::config::IndexPage>) -> Index {
    Index {
        site,
        slogan: index.slogan,
        title: index.title,
        skills: index.skills,
        github: index.github,
        mail: contact.mail,
        matrix: contact.matrix,
        threema: contact.threema,
    }
}


#[derive(Template)]
#[template(path = "contact.html")]
struct Contact {
    mail: String,
    matrix: String,
    threema: String,
    site: String,
    title: Option<String>,
}

async fn contact(
    Extension(details): Extension<app::config::Contact>, 
    Extension(site): Extension<String>, 
    Extension(index): Extension<app::config::IndexPage>,
) -> Contact {
    Contact {
        mail: details.mail.unwrap_or_default(),
        matrix: details.matrix.unwrap_or_default(),
        threema: details.threema.unwrap_or_default(),
        site,
        title: index.title,
    }
}

#[tokio::main]
async fn main() {
    let yaml_settings = "/etc/artemis/config.yml".to_string();
    let settings = match app::config::Config::from_file(yaml_settings) {
        Ok(setting) => setting,
        Err(e) => panic!("Could not load config file: {}", e),
    };

    // clone wiki repo to destination
    match app::git::clone_repository(
        &settings.content.wiki.repository,
        &settings.content.wiki.path,
    ) {
        Ok(_) => println!("Cloned repository"),
        Err(e) => {
            if e.code() != git2::ErrorCode::Exists {
                println!("{}", e.message());
                exit(1);
            }
        }
    };

    // clone blog repo to destination
    match app::git::clone_repository(
        &settings.content.blog.repository,
        &settings.content.blog.path,
    ) {
        Ok(_) => println!("Cloned repository"),
        Err(e) => {
            if e.code() != git2::ErrorCode::Exists {
                println!("{}", e.message());
                exit(1);
            }
        }
    };

    // ugly solution needs to be refined later
    let image_blog_path = settings
        .content
        .blog
        .path
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();
    let image_wiki_path = settings
        .content
        .wiki
        .path
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();


    // load initial post list
    let context_state = Arc::new(Mutex::new(handlers::post::ContextState {
        posts: handlers::post::load(&settings.content.blog.path).unwrap(),
        wiki: handlers::wiki::load(&settings.content.wiki.path).unwrap(),
        // Always set the blog path as first element and the wiki path as second element.
        repos: vec![settings.content.blog.path, settings.content.wiki.path],
        secret: settings.content.secret,
    }));

    let middleware = tower::ServiceBuilder::new()
        .layer(Extension(context_state))
        .layer(Extension(settings.contact))
        .layer(Extension(settings.server.host))
        .layer(Extension(settings.index));

    let app = Router::new()
        .route("/", get(index))
        .route("/pgp-key.txt", get(handlers::security::pgp_key))
        .route("/.well-known/:file", get(handlers::security::well_known))
        .route("/blog", get(handlers::blog::blog))
        // we should fix this through middleware
        .route("/blog/", get(handlers::blog::blog))
        .route("/blog/:title", get(handlers::blog::blog_post))
        .route("/wiki", get(handlers::wiki::wiki_posts))
        // we should fix this through middleware
        .route("/wiki/", get(handlers::wiki::wiki_posts))
        .route("/wiki/*title", get(handlers::wiki::wiki_posts))
        .route("/contact", get(contact))
        .route("/healthz", get(|| async { "health" }))
        .route("/update", post(handlers::update::update))
        .nest_service("/b/images",
            get_service(ServeDir::new(format!("{}/images", image_blog_path)).not_found_service(
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("unhandled server error"),
                ).into_service()
        )))
        .nest_service("/w/images",
            get_service(ServeDir::new(format!("{}/images", image_wiki_path)).not_found_service(
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("unhandled server error"),
                ).into_service()
        )))
        .nest_service("/static",ServeDir::new("./static").not_found_service(
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("unhandled server error"),
                ).into_service()
        ))
        .nest_service("/css",ServeDir::new("./css").not_found_service(
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("unhandled server error"),
                ).into_service()
        ))
        // global 404 fallback
        .fallback(handlers::status::code_404)
        .layer(middleware);


    let listen = &format!("{}:{}", settings.server.listen, settings.server.port);
    println!("A R T E M I S\nlistening on : {}", &listen);
    axum_server::bind(listen.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
