mod app;
mod components;
mod services;

use leptos::mount::mount_to_body;

fn main() {
    mount_to_body(app::App);
}
