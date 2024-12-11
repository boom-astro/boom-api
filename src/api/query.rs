use std::collections::HashMap;
use actix_web::{post, get, web, HttpResponse};
use mongodb::{bson::doc, Client, Collection};
use futures::TryStreamExt;

use crate::models::model::*;
use crate::models;

const DB_NAME: &str = "boom";

const SUPPORTED_QUERY_TYPES: [&str; 5] = ["find", "cone_search", "sample", "info", "count_documents"];
const SUPPORTED_INFO_COMMANDS: [&str; 4] = ["catalog_names", "catalog_info", "index_info", "db_info"];

async fn build_options(
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

#[get("/query/info")]
pub async fn get_catalog_names(client: web::Data<Client>, body: web::Json<InfoQueryBody>) -> HttpResponse {
    let command = body.command.clone().expect("command required for info query");
    if !SUPPORTED_INFO_COMMANDS.contains(&command.as_str()) {
        return HttpResponse::BadRequest().body(format!("Unknown info query type {command}"));
    }

    if command == "catalog_names" {
        // get collection names in alphabetical order
        let catalog_names = client.database(DB_NAME).list_collection_names().await.unwrap();
        let mut data = catalog_names
            .iter().filter(|name| !name.starts_with("system."))
            .collect::<Vec<&String>>();
        data.sort();
        return HttpResponse::Ok().json(data);
    } else if command == "catalog_info" {
        // get collection statistics for catalog(s)
        let catalogs = body.catalogs.clone().expect("catalog(s) is required for catalog info");
        let mut data = Vec::new();
        for name in catalogs {
            data.push(client.database(DB_NAME).run_command(doc! { "collstats": name}).await.unwrap());
        }
        return HttpResponse::Ok().json(data);
    } else if command == "index_info" {
        // get list of indexes on the collection
        let mut out_data = Vec::new();
        let catalogs = body.catalogs.clone().expect("catalog(s) is required for index_info");
        for i in 0..catalogs.len() {
            let collection: Collection<mongodb::bson::Document> = client.database(DB_NAME).collection(&catalogs[i]);
            let cursor = collection.list_indexes().await.unwrap();
            let data = cursor.try_collect::<Vec<mongodb::IndexModel>>().await.unwrap();
            out_data.push(data);
        }
        return HttpResponse::Ok().json(out_data);
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
    let collection: Collection<mongodb::bson::Document> = client.database(DB_NAME).collection(&catalog);

    let size = this_query.size.unwrap_or(1);
    if size > 1000 {
        return HttpResponse::BadRequest().body("size must be less than 1000");
    }
    let kwargs_sample = QueryKwargs {
        limit: Some(size),
        ..Default::default()
    };
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
    let collection: Collection<mongodb::bson::Document> = client.database(DB_NAME).collection(&catalog);
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
    let collection: Collection<mongodb::bson::Document> = client.database(DB_NAME).collection(&catalog);
    let cursor = collection.find(filter).with_options(find_options).await.unwrap();
    let docs = cursor.try_collect::<Vec<mongodb::bson::Document>>().await.unwrap();
    HttpResponse::Ok().json(docs)
}

/*
OLD:
{
  "query_type": "cone_search",
  "query": {
    "object_coordinates": {
      "cone_search_radius": 2,
      "cone_search_unit": "arcsec",
      "radec": {
        "object1": [
          71.6577756,
          -10.2263957
        ]
      }
    },
    "catalogs": {
      "ZTF_alerts": {
        "filter": {},
        "projection": {
          "_id": 0,
          "candid": 1,
          "objectId": 1
        }
      }
    }
  },
  "kwargs": {
    "filter_first": false
  }
}

NEW:

pub struct ConeSearchBody {
    pub radius: Option<f64>,
    pub unit: Option<Unit>,
    pub object_coordinates: Option<HashMap<String, [f64, 2]>>,
    pub catalogs: Option<HashMap<String, [Vec<bson::Document>, 2]>>,
    pub kwargs: Option<Kwargs>,
}

{
    
    "radius": 2,
    "unit": "arcsec",
    "object_coordinates": {
        "radec": [
            {"object1": [
                71.6577756,
                -10.2263957
            ]},
            {"object2": [
                23.91,
                -42.1
            ]}
        ]
    },
    "catalogs": [
        {
            "ZTF_alerts": {
                "filter": {},
                "projection": {
                    "_id": 0,
                    "candid": 1,
                    "objectId": 1
                }
            }
        },
        {
            "some_other_catalog": {
                "filter": {},
                "projection": {
                    "_id": 0,
                    "candid": 1,
                    "objectId": 1
                }
            }
        }
    ],
    "kwargs": {
        "filter_first": false
    }
}
*/

#[get("/query/cone_search")]
pub async fn cone_search(client: web::Data<Client>, body: web::Json<QueryBody>) -> HttpResponse {
    let this_query = body.query.clone().unwrap_or_default();
    let query_filter = this_query.filter
        .expect("filter is required for cone_search");
    let kwargs = body.kwargs.clone().unwrap_or_default();
    let catalog = this_query.catalog.expect("catalog required for cone_search");
    let projection = this_query.projection;
    let object_coordinates = this_query.object_coordinates
        .expect("object_coordinates is required for cone_serarch");
    let collection: Collection<mongodb::bson::Document> = client.database(DB_NAME).collection(&catalog);
    // Batch cone_search calls
    let mut docs: HashMap<String, Vec<mongodb::bson::Document>> = HashMap::new();
    let radius = object_coordinates.radius.expect("Radius required for cone_search");
    let unit = object_coordinates.unit.expect("Unit required for cone_search");
    let find_options = build_options(projection, kwargs).await;
    // loop through all coordinates in query
    for coord in object_coordinates.radec {
        // extract coordinates and name for map
        for key in coord.keys() {
            let radec = (coord[key][0], coord[key][1]);
            let filter = build_cone_search_filter(
                query_filter.clone(), 
                radec, 
                radius, 
                unit.clone()
            ).await;
            // perform cone_search on database
            let cursor = collection
                .find(filter)
                .with_options(find_options.clone()).await.unwrap();
            // create map entry for this object's cone search 
            docs.insert(
                key.to_string(), 
                cursor.try_collect::<Vec<mongodb::bson::Document>>().await.unwrap()
            );
        }
    }
    HttpResponse::Ok().json(docs)
}



// #[post("/query")]
// pub async fn query(client: web::Data<Client>, body: web::Json<QueryBody>) -> HttpResponse {
//     let query_type = body.query_type.as_str();
//     let query = body.query.clone().unwrap_or(Query{..Default::default()});
//     let kwargs = body.kwargs.clone().unwrap_or( QueryKwargs { ..Default::default()});
    
//     if !SUPPORTED_QUERY_TYPES.contains(&query_type) {
//         return HttpResponse::BadRequest().body(format!("Unknown query type {query_type}"));
//     }
    
//     if query_type == "info" {
//         let command = query.command.clone().expect("command is required for info queries");
//         if !SUPPORTED_INFO_COMMANDS.contains(&command.as_str()) {
//             return HttpResponse::BadRequest().body(format!("Uknown command {command}"));
//         }
//         if command == "catalog_names" {
//             // get collection names in alphabetical order
//             let catalog_names = client.database(DB_NAME).list_collection_names().await.unwrap();
//             let mut data = catalog_names
//                 .iter().filter(|name| !name.starts_with("system."))
//                 .collect::<Vec<&String>>();
//             data.sort();
//             return HttpResponse::Ok().json(data);
//         } else if command == "catalog_info" {
//             // get collection statistics
//             let catalog = query.catalog.expect("catalog is required for catalog_info");
//             let data = client.database(DB_NAME).run_command(doc! { "collstats": catalog }).await.unwrap();
//             return HttpResponse::Ok().json(data);
//         } else if command == "index_info" {
//             // get list of indexes on the collection
//             let catalog = query.catalog.expect("catalog is required for index_info");
//             let collection: Collection<mongodb::bson::Document> = client.database(DB_NAME).collection(&catalog);
//             let cursor = collection.list_indexes().await.unwrap();
//             let data = cursor.try_collect::<Vec<mongodb::IndexModel>>().await.unwrap();
//             return HttpResponse::Ok().json(data);
//         } else if command == "db_info" {
//             let data = client.database(DB_NAME).run_command(doc! { "dbstats": 1 }).await.unwrap();
//             return HttpResponse::Ok().json(data);
//         } else {
//             return HttpResponse::BadRequest().body(format!("Unkown command {command}"));
//         }
//     }
    
//     let catalog = query.catalog.unwrap();
//     let collection: Collection<mongodb::bson::Document> = client.database(DB_NAME).collection(&catalog);
    
//     if query_type == "sample" {
//         let size = query.size.unwrap_or(1);
//         if size > 1000 {
//             return HttpResponse::BadRequest().body("size must be less than 1000");
//         }
//         let kwargs_sample = QueryKwargs {
//             limit: Some(size),
//             ..Default::default()
//         };
//         // just use a find_one to get a single document as a sample of the collection
//         let options = build_options(None, kwargs_sample).await;
//         let cursor = 
//             collection.find(doc! {}).with_options(options).await.unwrap();
//         let docs = 
//             cursor.try_collect::<Vec<mongodb::bson::Document>>().await.unwrap();
//         HttpResponse::Ok().json(docs)
//     } else if query_type == "count_documents" {
//         let filter = query.filter.unwrap_or(doc!{});
//         let doc_count = collection.count_documents(filter).await;
//         match doc_count {
//             Err(e) => {
//                 HttpResponse::BadRequest().body(format!("bad request, got error {:?}", e))
//             },
//             Ok(x) => HttpResponse::Ok().json(x)
//         }
//     } else if query_type == "find" {
//         let filter = query.filter.expect("filter is required for find");
//         let projection = query.projection;
//         let find_options = build_options(projection, kwargs).await;
//         let cursor = collection.find(filter).with_options(find_options).await.unwrap();
//         let docs = cursor.try_collect::<Vec<mongodb::bson::Document>>().await.unwrap();
//         HttpResponse::Ok().json(docs)
//     } else if query_type == "cone_search" {
//         let query_filter = query.filter
//             .expect("filter is required for cone_search");
//         let projection = query.projection;
//         let object_coordinates = query.object_coordinates
//             .expect("object_coordinates is required for cone_serarch");
//         // Batch cone_search calls
//         let mut docs: HashMap<String, Vec<mongodb::bson::Document>> = HashMap::new();
//         let radius = object_coordinates.radius;
//         let unit = object_coordinates.unit.expect("Unit required for cone_search");
//         let find_options = build_options(projection, kwargs).await;
//         // loop through all coordinates in query
//         for coord in object_coordinates.radec {
//             // extract coordinates and name for map
//             for key in coord.keys() {
//                 let radec = (coord[key][0], coord[key][1]);
//                 let filter = build_cone_search_filter(
//                     query_filter.clone(), 
//                     radec, 
//                     radius, 
//                     unit.clone()
//                 ).await;
//                 // perform cone_search on database
//                 let cursor = collection
//                     .find(filter)
//                     .with_options(find_options.clone()).await.unwrap();
//                 // create map entry for this object's cone search 
//                 docs.insert(
//                     key.to_string(), 
//                     cursor.try_collect::<Vec<mongodb::bson::Document>>().await.unwrap()
//                 );
//             }
//         }
//         HttpResponse::Ok().json(docs)
//     } else {
//         HttpResponse::BadRequest().body(format!("Unknown query type {query_type}"))
//     }
// }