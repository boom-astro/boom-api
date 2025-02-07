
use std::ops::BitOrAssign;

use actix_web::{get, web, HttpResponse};
use mongodb::{bson::{doc, Document}, Client, Collection};
use futures::TryStreamExt;
use crate::models::alert_models;

const DB_NAME: &str = "boom";

// Get an object from a particular catalog by using its objectid
#[get("/alerts/get_object")]
pub async fn get_object(client: web::Data<Client>, body: web::Json<alert_models::GetObjectBody>) -> HttpResponse {
    let body = body.clone();
    let catalog = body.catalog.expect("catalog required for get_object");
    let collection: Collection<mongodb::bson::Document> = 
        client.database(DB_NAME).collection(&format!("{}_alerts", catalog));
    let object_id = body.object_id.expect("objectId required for get_object");

    // get brightest alert
    let find_options = mongodb::options::FindOptions::builder()
        .sort(doc! {
            "candidate.magpsf": 1,
        })
        .projection(
            doc!{ "_id": 0 }
        )
        .limit(1)
        .build();

    let mut brightest_alert_cursor = collection.find(doc! {
        "objectId": object_id.to_string(),
    }).with_options(find_options).await.expect("failed to execute find query");
    let brightest_alert = match brightest_alert_cursor.try_next().await {
        Ok(Some(alert)) => alert,
        Ok(None) => {
            return HttpResponse::Ok().body(format!("no object found with id {}", object_id));
        },
        Err(error) => {
            return HttpResponse::BadRequest().body(format!("error: {}", error));
        }
    };

    let find_options_alerts = mongodb::options::FindOptions::builder()
        .projection(
            doc! {
                "_id": 0,
                "cutoutScience": 0,
                "cutoutTemplate": 0,
                "curoutDifference": 0,
            }
        )
        .sort(
            doc! {
                "candidate.jd": 1,
            }
        )
        .build();

    let cursor_alerts = collection.find(doc!{
        "objectId": object_id.to_string(),
    }).with_options(find_options_alerts).await.unwrap();
    let alerts = cursor_alerts.try_collect::<Vec<mongodb::bson::Document>>().await.unwrap();

    let aux_collection_name = format!("{catalog}_alerts_aux");
    // then fetch the entry for that object in the aux collection
    let aux_collection: Collection<mongodb::bson::Document> = 
        client.database(DB_NAME).collection(aux_collection_name.as_str());
    let aux_entry = aux_collection.find_one(doc! {
        "_id": object_id.to_string(),
    }).await.expect("failed to exexcute find_one query").expect("no aux entry found");

    let mut data = doc! {};

    data.insert("objectId", object_id.to_string());
    // brightest alert has teh cutoutScience, cutoutTemplate, and cutoutDifference fields
    // that are byte strings
    match brightest_alert.get_document("cutoutScience") {
        Ok(cutout) => {
            data.insert("cutoutScience", cutout);
        },
        Err(error) => {
            println!("cutoutScience not found: {}", error);
            // print the type of the cutoutScience field
            data.insert("cutoutScience", "");
        }
    }
    data.insert("alerts", alerts);

    let prv_candidates = aux_entry.get_array("prv_candidates").unwrap();
    data.insert("prv_candidates", prv_candidates);

    // crossmatches is a Result<>, so we check if it's Ok or Err
    match aux_entry.get_document("cross_matches") {
        Ok(crossmatches) => {
            data.insert("cross_matches", crossmatches);
        },
        Err(_) => {
            data.insert("cross_matches", doc! {});
        }
    }

    HttpResponse::Ok().json(data)
}