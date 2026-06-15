#[cfg(feature = "frontend_resume_editor_svelte")]
pub mod proxy_handlers;
#[cfg(feature = "frontend_resume_editor_svelte")]
pub mod trailing_backslash_redirect;

#[cfg(feature = "frontend_resume_editor_svelte")]
pub fn mount_frontend_resume_editor_svelte(
    rocket: rocket::Rocket<rocket::Build>,
) -> rocket::Rocket<rocket::Build> {
    rocket
        .mount(
            "/",
            trailing_backslash_redirect::frontend_trailing_slash_redirect_routes(),
        )
        .mount(
            "/",
            routes![
                proxy_handlers::frontend_index_proxy_handler,
                proxy_handlers::frontend_proxy_handler,
                proxy_handlers::frontend_websocket_proxy_handler
            ],
        )
}

#[cfg(not(feature = "frontend_resume_editor_svelte"))]
pub fn mount_frontend_resume_editor_svelte(
    rocket: rocket::Rocket<rocket::Build>,
) -> rocket::Rocket<rocket::Build> {
    rocket
}
