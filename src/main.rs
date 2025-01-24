/*
API:
endpoints:
1. Query
    - cone_search   (performs a cone search based on the given data)
    - find          (mongodb:find operation)
    - sample        (given a catalog return one or more documents, without any filters)
    - info          (that can take a few commands including: list catalog names, 
                    give info/metadata about catalog, including info about indexes)
2. Filter
    - post_filter (posts a user's filter to the database)

3. alerts
    - get_object (copied from Theo's function)

    https://kowalski.caltech.edu/docs/api/#tag/queries
    Each of the query types are infered by the api upon receiving the request.
    The query type is located inside of a JSON object with string field "query_type".
    To submit a query, post to "/api/queries/" with the corresponding JSON payload.
    
*/
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
