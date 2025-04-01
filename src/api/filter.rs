use crate::models::filter_models::*;
use actix_web::{post, patch, web, HttpResponse};
use mongodb::{
    bson::{doc, Document},
    Client, Collection,
};

use super::util;

const DB_NAME: &str = "boom";

struct Filter {
    pub pipeline: Vec<mongodb::bson::Document>,
    pub permissions: Vec<i64>,
    pub catalog: String,
    pub id: i32,
}

// checks if a filter with the given id already exists in the database
async fn check_filter_exists(
    client: web::Data<Client>,
    filter_id: i32
) -> Result<bool, mongodb::error::Error> {
    let filter_collection = util::get_filter_collection(client, DB_NAME);
    let result = filter_collection
        .count_documents(doc!{ "filter_id": filter_id }).await;
    match result {
        Ok(count) => {
            return Ok(count > 0);
        }
        Err(e) => {
            return Err(e);
        }
    }
}

// prepends necessary portions to filter to run in database
fn build_test_filter(
    filter_catalog: String,
    filter_id: i32,
    filter_perms: Vec<i64>,
    mut filter_pipeline: Vec<Document>,
) -> Filter {
    let mut out_filter = vec![
        doc! {
            "$project": doc! {
                "cutoutScience": 0,
                "cutoutDifference": 0,
                "cutoutTemplate": 0,
                "publisher": 0,
                "schemavsn": 0
            }
        },
        doc! {
            "$lookup": doc! {
                "from": format!("{}_aux", filter_catalog),
                "localField": "objectId",
                "foreignField": "_id",
                "as": "aux"
            }
        },
        doc! {
            "$project": doc! {
                "objectId": 1,
                "candid": 1,
                "candidate": 1,
                "classifications": 1,
                "coordinates": 1,
                "cross_matches": doc! {
                    "$arrayElemAt": [
                        "$aux.cross_matches",
                        0
                    ]
                },
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
                                    "$in": [
                                        "$$x.programid",
                                        &filter_perms
                                    ]
                                },
                                {
                                    "$lt": [
                                        {
                                            "$subtract": [
                                                "$candidate.jd",
                                                "$$x.jd"
                                            ]
                                        },
                                        365
                                    ]
                                }
                            ]
                        }
                    }
                },
            }
        },
    ];
    out_filter.append(&mut filter_pipeline);
    let built_filt = Filter {
        pipeline: out_filter,
        permissions: filter_perms,
        catalog: filter_catalog,
        id: filter_id,
    };
    return built_filt;
}

// tests the functionality of a filter by running it on alerts in database
async fn test_run_filter(
    client: web::Data<Client>,
    catalog: String,
    filter: Filter,
) -> Result<(), mongodb::error::Error> {
    let collection: Collection<mongodb::bson::Document> = client
        .database(DB_NAME)
        .collection(format!("{}_alerts", catalog).as_str());

    let result = collection.aggregate(filter.pipeline).await;
    match result {
        Ok(_) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e);
        }
    }
}

// takes a verified filter and builds the properly formatted bson document for the database
fn build_filter_bson(filter: Filter) -> Result<mongodb::bson::Document, mongodb::error::Error> {
    // generate new object id
    let id = mongodb::bson::oid::ObjectId::new();
    let date_time = mongodb::bson::DateTime::now();
    let database_filter_bson = doc! {
        "_id": id,
        "group_id": 41, // consistent with other test filters
        "filter_id": filter.id,
        "catalog": filter.catalog,
        "permissions": filter.permissions,
        "active": true,
        "active_fid": filter.id,
        "fv": [
            // TODO: how to generate filter id's? ---> generate a random string (in future, hash pipeline)
            {
                "fid": "some_pipeline_id",
                "pipeline": filter.pipeline,
                "created_at": date_time,
            }
        ],
        "autosave": false,
        "update_annotations": true,
        "created_at": date_time,
        "last_modified": date_time,
    };
    Ok(database_filter_bson)
}

#[post("/filter")]
pub async fn post_filter(
    client: web::Data<Client>,
    body: web::Json<FilterSubmissionBody>,
) -> HttpResponse {
    let body = body.clone();
    // grab user filter
    let catalog = match body.catalog {
        Some(catalog) => catalog,
        None => {
            return HttpResponse::BadRequest().body("catalog not provided");
        }
    };
    let id = match body.id {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().body("filter id not provided");
        }
    };
    let permissions = match body.permissions {
        Some(permissions) => permissions,
        None => {
            return HttpResponse::BadRequest().body("permissions not provided");
        }
    };
    let pipeline = match body.pipeline {
        Some(pipeline) => pipeline,
        None => {
            return HttpResponse::BadRequest().body("pipeline not provided");
        }
    };

    // Test filter received from user
    // create production version of filter
    let filter = build_test_filter(catalog.clone(), id, permissions.clone(), pipeline.clone());
    // perform test run to ensure no errors
    match test_run_filter(client.clone(), catalog.clone(), filter).await {
        Ok(()) => {}
        Err(e) => {
            return HttpResponse::BadRequest().body(
                format!("Invalid filter submitted, filter test failed with error: {}", e));
        }
    }

    // save original filter to database
    let filter_collection: Collection<mongodb::bson::Document> =
        client.database(DB_NAME).collection("filters");
    let database_filter = Filter {
        pipeline,
        permissions,
        catalog,
        id,
    };
    let filter_bson = match build_filter_bson(database_filter) {
        Ok(bson) => bson,
        Err(e) => {
            return HttpResponse::BadRequest().body(format!("unable to create filter bson, got error: {}", e));
        }
    };
    match filter_collection.insert_one(filter_bson).await {
        Ok(_) => {
            return HttpResponse::Ok().body("successfully submitted filter to database");
        }
        Err(e) => {
            return HttpResponse::BadRequest().body(format!(
                "failed to insert filter into database. error: {}", 
                e
            ));
        }
    }
}

