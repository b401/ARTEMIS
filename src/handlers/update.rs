use crate::app::git::pull_repository;
use crate::handlers::{
    post::{load, ContextState},
    wiki::load as wikiLoad,
};
use axum::extract::Request;
use axum::{
    async_trait,
    extract::{Extension, FromRequestParts},
    http::{request::Parts, HeaderMap, StatusCode},
};
use axum_macros::debug_handler;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

const X_HUB_SIGNATURE_256: &str = "x-hub-signature-256";
#[derive(Deserialize, Debug)]
pub struct GithubSecret(String);

#[async_trait]
impl<S> FromRequestParts<S> for GithubSecret
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        github_secret(&parts.headers).map(Self).ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Can't determine authentication header",
        ))
    }
}

fn verify_signature(secret: &str, body: &[u8], github_sig: &str) -> bool {
    let digest: String = compute_digest(secret, body);
    digest.eq_ignore_ascii_case(github_sig)
}

fn compute_digest(secret: &str, data: &[u8]) -> String {
    use hmac::Mac;
    let mut digest = hmac::Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes())
        .expect("Failed to create hmac");
    digest.update(data);
    let digest = digest.finalize();
    to_hex(&digest.into_bytes())
}

fn to_hex(data: &[u8]) -> String {
    let mut output = String::with_capacity(data.len() * 2);
    for &byte in data {
        output.push(std::char::from_digit(byte as u32 >> 4, 16).unwrap());
        output.push(std::char::from_digit(byte as u32 & 0x0F, 16).unwrap());
    }
    output
}

fn github_secret(headers: &HeaderMap) -> Option<String> {
    headers
        .get(X_HUB_SIGNATURE_256)
        .and_then(|hv| hv.to_str().unwrap().split('=').nth(1))
        .and_then(|s| s.trim().parse::<String>().ok())
}

#[debug_handler]
pub async fn update(
    GithubSecret(user_agent): GithubSecret,
    Extension(context): Extension<Arc<Mutex<ContextState>>>,
    request: Request,
) -> StatusCode {
    let body = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(body) => body,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    let mut cnt = context.lock().expect("could not lock mutex");
    let mut verifier = Vec::with_capacity(cnt.repos.len());
    println!("Got a new pull request...");
    verifier.push(
        match verify_signature(&cnt.secret, body.as_ref(), &user_agent) {
            true => {
                for (i, repo) in cnt.repos.clone().iter().enumerate() {
                    match pull_repository(repo) {
                        Err(e) => {
                            println!(
                                "Failed to load repo: {}. \n Reason: {}",
                                &repo.to_str().unwrap_or("dunno"),
                                e.message()
                            );
                        }
                        _ => {
                            if i == 0 {
                                println!("Loading new blog pages!");
                                cnt.posts = load(repo).unwrap();
                            } else {
                                println!("Loading new wiki pages!");
                                cnt.wiki = wikiLoad(repo).unwrap();
                            }
                        }
                    };
                }
                true
            }
            false => false,
        },
    );

    if verifier.contains(&true) {
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    }
}
