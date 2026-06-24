use std::path::PathBuf;

use rocket::fs::{FileServer, Options};

const API_PLAYGROUND_DIST_ENV_VAR: &str = "API_PLAYGROUND_DIST_PATH";

/// Mounts the API Playground static file server at `/api_playground`.
///
/// Reads the dist directory location from the `API_PLAYGROUND_DIST_PATH`
/// environment variable. If the variable is missing or the path is invalid,
/// a warning is logged and `/api_playground` is not mounted.
///
/// The `Options::Index` flag ensures that requests to `/api_playground`
/// (or `/api_playground/`) serve `index.html`.
pub fn mount_api_playground(
    rocket: rocket::Rocket<rocket::Build>,
) -> rocket::Rocket<rocket::Build> {
    let dist_path = match std::env::var(API_PLAYGROUND_DIST_ENV_VAR) {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            log::warn!(
                "{} is not set; skipping /api_playground mount.",
                API_PLAYGROUND_DIST_ENV_VAR
            );
            return rocket;
        }
    };

    if !dist_path.exists() {
        log::warn!(
            "API Playground dist directory was not found at {}. Skipping /api_playground mount.",
            dist_path.display()
        );
        return rocket;
    }

    if !dist_path.is_dir() {
        log::warn!(
            "API Playground dist path is not a directory: {}. Skipping /api_playground mount.",
            dist_path.display()
        );
        return rocket;
    }

    let index_html = dist_path.join("index.html");
    if !index_html.exists() {
        log::warn!(
            "API Playground dist is missing required file: {}. Skipping /api_playground mount.",
            index_html.display()
        );
        return rocket;
    }

    rocket.mount(
        "/api_playground",
        FileServer::new(dist_path, Options::Index),
    )
}
