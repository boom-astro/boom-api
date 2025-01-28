use std::collections::HashMap;
use actix_web::{get, web, HttpResponse};
use mongodb::{bson::{doc, Document}, Client, Collection, IndexModel};
use futures::TryStreamExt;

use crate::models::query_models::*;

const DB_NAME: &str = "boom";

const SUPPORTED_QUERY_TYPES: [&str; 5] = ["find", "cone_search", "sample", "info", "count_documents"];
const SUPPORTED_INFO_COMMANDS: [&str; 4] = ["catalog_names", "catalog_info", "index_info", "db_info"];

pub async fn build_options(
    projection: Option<mongodb::bson::Document>, 
    kwargs: QueryKwargs
) -> mongodb::options::FindOptions {
    let mut find_options = mongodb::options::FindOptions::default();
    
    if kwargs.limit.is_some() {
        find_options.limit = Some(kwargs.limit.unwrap());
    }
    if kwargs.skip.is_some() {
        find_options.skip = Some(kwargs.skip.unwrap());
    }
    if kwargs.sort.is_some() {
        find_options.sort = Some(kwargs.sort.unwrap());
    }
    if kwargs.max_time_ms.is_some() {
        find_options.max_time = Some(std::time::Duration::from_millis(kwargs.max_time_ms.unwrap()));
    }
    if projection.is_some() {
        find_options.projection = Some(projection.unwrap());
    }

    find_options
}

async fn build_cone_search_filter(
    mut filter: mongodb::bson::Document,
    radec: (f64, f64),
    mut radius: f64,
    unit: Unit,
) -> mongodb::bson::Document {
    
    let ra = radec.0 - 180.0;
    let dec = radec.1;

    // convert radius to radians based on unit
    match unit {
        Unit::Degrees => radius = radius.to_radians(),
        Unit::Arcseconds => radius = radius.to_radians() / 3600.0,
        Unit::Arcminutes => radius = radius.to_radians() / 60.0,
        Unit::Radians => {}
    }

    let center_sphere = doc! {
        "$centerSphere": [[ra, dec], radius]
    };

    let geo_within = doc! {
        "$geoWithin": center_sphere
    };

    filter.insert("coordinates.radec_geojson", geo_within);

    filter
}


pub async fn get_catalog_names(client: web::Data<Client>, db_name: &str) -> Vec<String> {
    // get collection names in alphabetical order
    let collection_names = client.database(db_name).list_collection_names().await.unwrap();
    let mut data = collection_names
        .iter().filter(|name| !name.starts_with("system.")).cloned()
        .collect::<Vec<String>>();
    data.sort();
    return data;
}

pub async fn get_catalog_info(catalogs: Vec<String>, client: web::Data<Client>, db_name: &str) -> Vec<Document> {
    let mut data = Vec::new();
    for catalog in catalogs {
        data.push(
            client
            .database(db_name)
            .run_command(doc! { "collstats": format!("{}_alerts", catalog)}).await.unwrap());
    }
    return data;
}

pub async fn get_index_info(client: web::Data<Client>, catalogs: Vec<String>, db_name: &str) -> Vec<Vec<IndexModel>> {
    let mut out_data = Vec::new();
    for i in 0..catalogs.len() {
        let collection: Collection<mongodb::bson::Document> = 
            client.database(db_name).collection(&format!("{}_alerts", &catalogs[i]));
        let cursor = collection.list_indexes().await.unwrap();
        let data = cursor.try_collect::<Vec<mongodb::IndexModel>>().await.unwrap();
        out_data.push(data);
    }
    return out_data;
}

#[get("/query/info")]
pub async fn get_info(client: web::Data<Client>, body: web::Json<InfoQueryBody>) -> HttpResponse {
    let command = body.command.clone().expect("command required for info query");
    if !SUPPORTED_INFO_COMMANDS.contains(&command.as_str()) {
        return HttpResponse::BadRequest().body(format!("Unknown info query type {command}"));
    }
    if command == "catalog_names" { // get collection names in alphabetical order
        let data = get_catalog_names(client, DB_NAME).await;
        return HttpResponse::Ok().json(data);
    } else if command == "catalog_info" { // get collection statistics for catalog(s)
        let catalogs = body.catalogs.clone().expect("catalog(s) is required for catalog info");
        let data = get_catalog_info(catalogs, client, DB_NAME).await;
        return HttpResponse::Ok().json(data);
    } else if command == "index_info" { // get list of indexes on the collection
        let catalogs = body.catalogs.clone().expect("catalog(s) is required for index_info");
        let data = get_index_info(client, catalogs, DB_NAME).await;
        return HttpResponse::Ok().json(data);
    } else if command == "db_info" {
        let data = client.database(DB_NAME).run_command(doc! { "dbstats": 1 }).await.unwrap();
        return HttpResponse::Ok().json(data);
    } else {
        return HttpResponse::BadRequest().body(format!("Unkown command {command}"));
    }
}

#[get("/query/sample")]
pub async fn sample(client: web::Data<Client>, body: web::Json<QueryBody>) -> HttpResponse {
    let this_query = body.query.clone().unwrap_or_default();
    let catalog = this_query.catalog.unwrap();
    let collection: Collection<mongodb::bson::Document> = 
        client.database(DB_NAME).collection(&format!("{}_alerts", catalog));

    let size = this_query.size.unwrap_or(1);
    if size > 1000 {
        return HttpResponse::BadRequest().body("size must be less than 1000");
    }
    let kwargs_sample = QueryKwargs {
        limit: Some(size),
        ..Default::default()
    };
    println!("{:?}", kwargs_sample);
    // use find to get a sample of the collection
    let options = build_options(None, kwargs_sample).await;
    let cursor = 
        collection.find(doc! {}).with_options(options).await.unwrap();
    let docs = 
        cursor.try_collect::<Vec<mongodb::bson::Document>>().await.unwrap();
    HttpResponse::Ok().json(docs)
}

#[get("/query/count_documents")]
pub async fn count_documents(client: web::Data<Client>, body: web::Json<QueryBody>) -> HttpResponse {
    let this_query = body.query.clone().unwrap_or_default();
    let catalog = this_query.catalog.unwrap();
    let collection: Collection<mongodb::bson::Document> = 
        client.database(DB_NAME).collection(&format!("{}_alerts", catalog));
    let filter = this_query.filter.unwrap_or(doc!{});
    let doc_count = collection.count_documents(filter).await;
    match doc_count {
        Err(e) => {
            HttpResponse::BadRequest().body(format!("bad request, got error {:?}", e))
        },
        Ok(x) => HttpResponse::Ok().json(x)
    }
}

#[get("/query/find")]
pub async fn find(client: web::Data<Client>, body: web::Json<QueryBody>) -> HttpResponse {
    let this_query = body.query.clone().unwrap_or_default();
    let filter = this_query.filter.expect("filter is required for find");
    let catalog = this_query.catalog.expect("catalog is required for find");
    let find_options = build_options(
        this_query.projection, body.kwargs.clone().unwrap_or_default()
    ).await;
    let collection: Collection<mongodb::bson::Document> = 
        client.database(DB_NAME).collection(&format!("{}_alerts", catalog));
    let cursor = collection.find(filter).with_options(find_options).await.unwrap();
    let docs = cursor.try_collect::<Vec<mongodb::bson::Document>>().await.unwrap();
    HttpResponse::Ok().json(docs)
}

#[get("/query/cone_search")]
pub async fn cone_search(client: web::Data<Client>, body: web::Json<ConeSearchBody>) -> HttpResponse {
    let radius = body.radius.expect("Radius required for cone search");
    let unit = body.unit.clone().expect("Unit required for cone search");
    let object_coordinates = body
        .clone().object_coordinates
        .expect("Object coordinates required for cone_search");

    let catalog_details = body.catalog.clone().expect("Catalog(s) required for cone_search");
    let catalog_name = catalog_details.catalog_name.expect("catalog_name required for a catalog");
    let collection: Collection<mongodb::bson::Document> = 
        client.database(DB_NAME).collection(&format!("{}_alerts", catalog_name));

    let projection = catalog_details.projection;
    let input_filter = catalog_details.filter.unwrap_or(doc! {});

    let kwargs = body.kwargs.clone().unwrap_or_default();
    let find_options = build_options(projection, kwargs).await;
    
    let mut docs: HashMap<String, Vec<mongodb::bson::Document>> = HashMap::new();
    for (object_name, radec) in object_coordinates {
        let filter = build_cone_search_filter(
            input_filter.clone(),
            (radec[0], radec[1]),
            radius,
            unit.clone()
        ).await;
        // perform cone_search on database
        let cursor = collection
            .find(filter)
            .with_options(find_options.clone()).await.unwrap();
        // create map entry for this object's cone search
        docs.insert(
            object_name,
            cursor.try_collect::<Vec<mongodb::bson::Document>>().await.unwrap()
        );
    }
    HttpResponse::Ok().json(docs)
}
