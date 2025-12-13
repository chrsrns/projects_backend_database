#[macro_use]
extern crate rocket;

use api;

#[launch]
fn rocket() -> _ {
    api::build_rocket()
}
