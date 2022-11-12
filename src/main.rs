use askama_axum::Template;
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
    extract::Extension,
    handler::Handler,
    http::StatusCode,
    routing::{get, get_service, post},
    Router,
};
use std::sync::{Arc, Mutex};
use tower_http::services::ServeDir;

#[derive(Template)]
#[template(path = "index.html")]
struct Index {}

#[derive(Template)]
#[template(path = "contact.html")]
struct Contact {
    mail: String,
    matrix: String,
    threema: String,
}

async fn contact(Extension(details): Extension<app::config::Contact>) -> Contact {
    Contact {
        mail: details.mail,
        matrix: details.matrix,
        threema: details.threema,
    }
}

async fn index() -> Index {
    Index {}
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
        .layer(Extension(settings.contact));

    let app = Router::new()
        .route("/", get(index))
        .route("/pgp-key.txt", get(handlers::security::pgp_key))
        .route("/.well-known/:file", get(handlers::security::well_known))
        .route("/blog", get(handlers::blog::blog))
        .route("/blog/:title", get(handlers::blog::blog_post))
        .route("/wiki/*title", get(handlers::wiki::wiki_posts))
        .route("/contact", get(contact))
        .route("/healthz", get(|| async { "health" }))
        .route("/update", post(handlers::update::update))
        .nest(
            "/b/images",
            get_service(ServeDir::new(format!("{}/images", image_blog_path))).handle_error(
                |err: std::io::Error| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("unhandled server error: {}", err),
                    )
                },
            ),
        )
        .nest(
            "/w/images",
            get_service(ServeDir::new(format!("{}/images", image_wiki_path))).handle_error(
                |err: std::io::Error| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("unhandled server error: {}", err),
                    )
                },
            ),
        )
        .nest(
            "/static",
            get_service(ServeDir::new("./static")).handle_error(|err: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("unhandled server error: {}", err),
                )
            }),
        )
        .nest(
            "/css",
            get_service(ServeDir::new("./css")).handle_error(|err: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("unhandled server error: {}", err),
                )
            }),
        )
        .layer(middleware);

    // global 404 fallback
    let app = app.fallback(handlers::status::code_404.into_service());

    let listen = &format!("{}:{}", settings.server.listen, settings.server.port);
    println!("A R T E M I S\nlistening on : {}", &listen);
    axum::Server::bind(&listen.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
