mod api;
mod todos;

use std::sync::Mutex;

use actix_web::{App, HttpServer};
// use chrono::{NaiveDateTime, NaiveDate, NaiveTime};
// use todos::{Todo, date::Due};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // let todo = Todo {
    //     name: "Date".to_string(),
    //     description: Some("This todo has a date due".to_string()),
    //     due: Some(Due { time: (NaiveDateTime::new(NaiveDate::from_ymd(2022, 10, 1), NaiveTime::from_hms(0, 0, 0)), None), repeat: false, hours: None, days: None, weeks: None, months: None, log: vec![] }),
    //     priority: 10,
    //     done: false,
    //     tags: vec!["date".to_string(), "test".to_string()],
    //     overdue: false,
    //     index: 0,
    // };
    // let mut profiles = todos::Profiles::new();
    // let profile = profiles.get_current_mut();
    // profile.add_todo(todo, 0);
    // profiles.save();

    let state = actix_web::web::Data::new(Mutex::new(todos::Profiles::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(api::get_api)
            .service(api::post_api)
            .service(api::get_api_current)
            .service(api::put_api_current)
            .service(api::get_api_idx)
            .service(api::put_api_idx)
            .service(api::delete_api_idx)
            .service(api::get_api_idx_name)
            .service(api::put_api_idx_name)
            .service(api::get_api_idx_todos)
            .service(api::post_api_idx_todos)
            .service(api::delete_api_idx_todos_done)
            .service(api::get_api_idx_todos_idx1)
            .service(api::put_api_idx_todos_idx1)
            .service(api::delete_api_idx_todos_idx1)
            .service(api::get_api_idx_links)
            .service(api::post_api_idx_links)
            .service(api::get_api_idx_links_idx1)
            .service(api::post_api_idx_links_idx1)
            .service(api::delete_api_idx_links_idx1)
            .service(api::get_api_idx_links_idx1_name)
            .service(api::put_api_idx_links_idx1_name)
            .service(api::get_api_idx_links_idx1_path)
            .service(api::put_api_idx_links_idx1_path)
            .service(actix_files::Files::new("/", "/home/m3/my-stuff/Projects/Svelte/lunite-web/public/").show_files_listing().index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
