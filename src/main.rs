#![forbid(unsafe_code)]
extern crate chrono;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;

mod api;

#[launch]
fn run() -> _ {
    let api_base = "/api/v1/";
    rocket::build()
        .mount(api_base, api::routes())
        .register(api_base, api::catchers())
}
