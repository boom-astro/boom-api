use crate::{models::response, api::util};
use actix_web::{get, web, HttpResponse};
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Document}, Client, Collection
};

const DB_NAME: &str = "boom";

// TODO: check notion
/*
1. get the most recent detection for the object
    sort alerts by jd in ascending order
2. get the image information for that detection
    get the cutoutScience, cutoutTemplate, and cutoutDifference fields
3. get metadata for that detection
    ...
4. get the crossmatches and prv_candidates for that object from the aux table
*/

#[get("/alerts/{survey_name}/get_object/{object_id}")]
pub async fn get_object(
    client: web::Data<Client>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (survey_name, object_id) = path.into_inner();
    let survey_name = survey_name.to_uppercase(); // TEMP to match with "ZTF"

    let alerts_collection: Collection<mongodb::bson::Document> = client
        .database(DB_NAME)
        .collection(&format!("{}_alerts", survey_name));

    let aux_collection: Collection<Document> = client
        .database(DB_NAME)
        .collection(&format!("{}_alerts_aux", survey_name));
    
    // find options for getting most recent alert from alerts collection
    let find_options_recent = mongodb::options::FindOptions::builder()
        .sort(doc! {
            "candidate.jd": 1,
        })
        .projection(doc! {
            "_id": 1,
            "candidate": 1,
            "cutoutScience": 1,
            "cutoutTemplate": 1,
            "cutoutDifference": 1,
        })
        .build();

    // get the most recent alert for the object
    let mut alert_cursor = match alerts_collection
        .find(doc! {
            "objectId": object_id.clone(),
        })
        .with_options(find_options_recent)
        .await {
            Ok(cursor) => cursor,
            Err(error) => {
                return response::internal_error(&format!("error getting documents: {}", error));
            }
        };
    let newest_alert = match alert_cursor
        .try_next()
        .await {
            Ok(Some(alert)) => alert,
            Ok(None) => {
                return response::ok(&format!("no object found with id {}", object_id), serde_json::Value::Null);
            },
            Err(error) => {
                return response::internal_error(&format!("error getting documents: {}", error));
            }
        };

    let find_options_aux = mongodb::options::FindOneOptions::builder()
        .projection(doc! {
            "_id": 0,
            "prv_candidates": 1,
            "cross_matches": 1,
        })
        .build();
    
    // get crossmatches and light curve data from aux collection
    let aux_entry = match aux_collection
        .find_one(doc! {
            "_id": object_id.clone(),
        })
        .with_options(find_options_aux)
        .await {
            Ok(entry) => {
                match entry {
                    Some(doc) => doc,
                    None => {
                        return response::ok("no aux entry found", serde_json::Value::Null);
                    }
                }
            },
            Err(error) => {
                return response::internal_error(&format!("error getting documents: {}", error));
            }
        };

    let mut data = doc!{};
    // organize response
    data.insert("objectId", object_id.clone());
    data.insert("alert metadata", newest_alert.get_document("candidate").unwrap());
    data.insert("cutoutScience", newest_alert.get_document("cutoutScience").unwrap());
    data.insert("cutoutTemplate", newest_alert.get_document("cutoutTemplate").unwrap());
    data.insert("cutoutDifference", newest_alert.get_document("cutoutDifference").unwrap());
    data.insert("prv_candidates", aux_entry.get_array("prv_candidates").unwrap());
    data.insert("cross_matches", aux_entry.get_document("cross_matches").unwrap());
    
    return response::ok(&format!("object found with object_id: {}", object_id), serde_json::json!(data));
}
