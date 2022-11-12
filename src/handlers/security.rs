use crate::handlers::status;
use axum::extract::Path;
use std::fs;

pub async fn pgp_key() -> Result<String, status::ErrorHandler> {
    match &fs::read("/etc/artemis/pgp-key.txt") {
        Ok(f) => Ok(String::from_utf8_lossy(f).parse().unwrap()),
        Err(_) => Err(status::code_generic(404).await),
    }
}

pub async fn well_known(Path(file): Path<String>) -> Result<String, status::ErrorHandler> {
    // TODO
    // need to fix this :D
    if file.as_str() == "security.txt" {
        match &fs::read(format!("/etc/artemis/well-known/{}", file)) {
            Ok(f) => Ok(String::from_utf8_lossy(f).parse().unwrap()),
            Err(_) => Err(status::code_generic(500).await),
        }
    } else {
        Err(status::code_generic(418).await)
    }
}
