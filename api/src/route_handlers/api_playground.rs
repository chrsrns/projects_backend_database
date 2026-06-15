use std::path::PathBuf;

use rocket::fs::{FileServer, Options};

/// Mounts the API Playground static file server at `/api_playground`.
///
/// Serves the compiled Leptos WASM application from `api_playground/dist/`.
/// The `Options::Index` flag ensures that requests to `/api_playground`
/// (or `/api_playground/`) serve `index.html`.
pub fn mount_api_playground(rocket: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
    let dist_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("api_playground")
        .join("dist");

    rocket.mount("/api_playground", FileServer::new(dist_path, Options::Index))
}
