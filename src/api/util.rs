use actix_web::web;
use mongodb::{bson::Document, Client, Collection};

// TODO: add check for valid catalogs
// retrieves a mongodb collection
pub fn get_collection(
    client: web::Data<Client>,
    catalog_name: &str,
    db_name: &str,
) -> Collection<Document> {
    let collection: Collection<mongodb::bson::Document> =
        client.database(db_name).collection(catalog_name);
    collection
}

// TODO: add check for valid catalogs
pub fn get_filter_collection(client: web::Data<Client>, db_name: &str) -> Collection<Document> {
    let collection: Collection<Document> = client.database(db_name).collection("filters");
    collection
}
