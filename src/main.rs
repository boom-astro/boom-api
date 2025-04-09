mod api;
mod models;

use actix_web::{web, App, HttpServer};
use mongodb::Client;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let user = "mongoadmin";
    let pass = "mongoadminsecret";
    let uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| format!("mongodb://{user}:{pass}@localhost:27017").into());
    let client = Client::with_uri_str(uri).await.expect("failed to connect");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(api::query::get_info)
            .service(api::query::sample)
            .service(api::query::cone_search)
            .service(api::query::count_documents)
            .service(api::query::find)
            .service(api::alerts::get_object)
            .service(api::filter::post_filter)
            .service(api::filter::add_filter_version)
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await
}

// mod query;

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     let user = "mongoadmin";
//     let pass = "mongoadminsecret";
//     let uri = std::env::var("MONGODB_URI")
//         .unwrap_or_else(|_| format!("mongodb://{user}:{pass}@localhost:27017").into());
//     let client = Client::with_uri_str(uri).await.expect("failed to connect");

//     HttpServer::new(move || {
//         App::new()
//             .app_data(web::Data::new(client.clone()))
//             .service(query::query)
//     })
//     .bind(("0.0.0.0", 4000))?
//     .run()
//     .await
// }
