use crate::models::response;
use actix_web::{get, web, HttpResponse};
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    Client, Collection,
};

const DB_NAME: &str = "boom";

#[derive(serde::Deserialize, serde::Serialize)]
struct PrvCandidate {
    jd: f64,
    band: String,
    magpsf: f64,
    sigmapsf: f64,
    diffmaglim: f64,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct PrvNondetection {
    jd: f64,
    band: String,
    diffmaglim: f64,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Alert {
    candid: i64,
    object_id: String,
    candidate: Document,
    prv_candidates: Vec<PrvCandidate>,
    prv_nondetections: Vec<PrvNondetection>,
    classifications: Document,
    cutout_science: Vec<u8>,
    cutout_template: Vec<u8>,
    cutout_difference: Vec<u8>,
}

#[get("/alerts/{survey_name}/{object_id}")]
pub async fn get_object(
    client: web::Data<Client>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (survey_name, object_id) = path.into_inner();
    let survey_name = survey_name.to_uppercase(); // TEMP to match with "ZTF"

    let db = client.database(DB_NAME);

    let alerts_collection: Collection<Document> = db.collection(&format!("{}_alerts", survey_name));

    // first we find the candid (_id) of the latest alert with that object_id
    let latest_alert_candid = match alerts_collection
        .find(doc! {
            "objectId": object_id.clone(),
        })
        .projection(doc! {
            "_id": 1,
        })
        .sort(doc! {
            "candidate.jd": -1,
        })
        .limit(1)
        .await
    {
        Ok(mut cursor) => match cursor.try_next().await {
            Ok(Some(doc)) => doc.get_i64("_id").unwrap(),
            Ok(None) => {
                return response::ok(
                    &format!("no object found with id {}", object_id),
                    serde_json::Value::Null,
                );
            }
            Err(error) => {
                return response::internal_error(&format!("error getting documents: {}", error));
            }
        },
        Err(error) => {
            return response::internal_error(&format!("error getting documents: {}", error));
        }
    };

    let alerts_collection: Collection<Alert> = db.collection(&format!("{}_alerts", survey_name));

    let mut alert_cursor = alerts_collection
        .aggregate(vec![
            doc! {
                "$match": {
                    "_id": latest_alert_candid,
                }
            },
            doc! {
                "$project": {
                    "objectId": 1,
                    "candidate": 1,
                    "classifications": 1,
                }
            },
            doc! {
                "$lookup": {
                    "from": "ZTF_alerts_aux",
                    "localField": "objectId",
                    "foreignField": "_id",
                    "as": "aux"
                }
            },
            doc! {
                "$lookup": {
                    "from": "ZTF_alerts_cutouts",
                    "localField": "_id",
                    "foreignField": "_id",
                    "as": "object"
                }
            },
            doc! {
                "$project": doc! {
                    "objectId": 1,
                    "candidate": 1,
                    "classifications": 1,
                    "prv_candidates": doc! {
                        "$filter": doc! {
                            "input": doc! {
                                "$arrayElemAt": [
                                    "$aux.prv_candidates",
                                    0
                                ]
                            },
                            "as": "x",
                            "cond": doc! {
                                "$and": [
                                    {
                                        "$gte": [
                                            {
                                                "$subtract": [
                                                    "$candidate.jd",
                                                    "$$x.jd"
                                                ]
                                            },
                                            0
                                        ]
                                    },

                                ]
                            }
                        }
                    },
                    "prv_nondetections": doc! {
                        "$filter": doc! {
                            "input": doc! {
                                "$arrayElemAt": [
                                    "$aux.prv_nondetections",
                                    0
                                ]
                            },
                            "as": "x",
                            "cond": doc! {
                                "$and": [
                                    {
                                        "$gte": [
                                            {
                                                "$subtract": [
                                                    "$candidate.jd",
                                                    "$$x.jd"
                                                ]
                                            },
                                            0
                                        ]
                                    },

                                ]
                            }
                        }
                    },
                    "cutoutScience": doc! {
                        "$arrayElemAt": [
                            "$object.cutoutScience",
                            0
                        ]
                    },
                    "cutoutTemplate": doc! {
                        "$arrayElemAt": [
                            "$object.cutoutTemplate",
                            0
                        ]
                    },
                    "cutoutDifference": doc! {
                        "$arrayElemAt": [
                            "$object.cutoutDifference",
                            0
                        ]
                    }
                }
            },
            doc! {
                "$project": doc! {
                    "objectId": 1,
                    "candidate": 1,
                    "classifications": 1,
                    "prv_candidates.jd": 1,
                    "prv_candidates.magpsf": 1,
                    "prv_candidates.sigmapsf": 1,
                    "prv_candidates.band": 1,
                    "prv_candidates.diffmaglim": 1,
                    "prv_nondetections.jd": 1,
                    "prv_nondetections.band": 1,
                    "prv_nondetections.diffmaglim": 1,
                    "cutoutScience": 1,
                    "cutoutTemplate": 1,
                    "cutoutDifference": 1
                }
            },
        ])
        .await
        .unwrap();

    let alert = match alert_cursor.try_next().await {
        Ok(Some(alert)) => alert,
        Ok(None) => {
            return response::ok(
                &format!("no object found with id {}", object_id),
                serde_json::Value::Null,
            );
        }
        Err(error) => {
            return response::internal_error(&format!("error getting documents: {}", error));
        }
    };

    return response::ok(
        &format!("object found with object_id: {}", object_id),
        serde_json::json!(alert),
    );
}
