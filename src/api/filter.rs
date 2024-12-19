use mongodb::{bson::{doc, Document}, Client, Collection};
use actix_web::{post, web, HttpResponse};
use std::{borrow::Borrow, error::Error};
use crate::api::query;
use crate::models::filter_models::*;



const DB_NAME: &str = "boom";

struct Filter {
    pub pipeline: Vec<mongodb::bson::Document>,
    pub permissions: Vec<i64>,
    pub catalog: String,
    pub id: i32,
}

// prepends necessary portions to filter to run in database
fn build_test_filter(
    filter_catalog: String, 
    filter_id: i32, 
    filter_perms: Vec<i64>, 
    mut filter_pipeline: Vec<Document>
) -> Filter {
    let mut out_filter = vec![
        doc! {
            "$match": doc! {
                // during filter::run proper candis are inserted here
            }
        },
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
        }
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
async fn test_filter(client: web::Data<Client>, catalog: String, filter: Filter) -> Result<(), mongodb::error::Error> {
    let collection: Collection<mongodb::bson::Document> = 
        client.database(DB_NAME).collection(format!("{}_alerts", catalog).as_str());
    
    let result = collection.aggregate(filter.pipeline).await;
    match result {
        Ok(_) => {
            return Ok(());
        },
        Err(e) => {
            return Err(e);
        }
    }
}

// takes tested filter and builds the properly formatted bson document for the database
fn build_filter_bson(filter: Filter) -> Result<mongodb::bson::Document, mongodb::error::Error> {
    // generate new object id
    let id = mongodb::bson::oid::ObjectId::new();
    let date_time = mongodb::bson::DateTime::now();
    // TODO: how to manage multiple filters in each FILTER within the database
    let database_filter_bson = doc! {
        "_id": id,
        "group_id": 41, // consistent with other test filters
        "filter_id": filter.id,
        "catalog": filter.catalog,
        "permissions": filter.permissions,
        "active": true,
        "active_fid": filter.id,
        "fv": [
            // TODO: how to generate pipeline id's?
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

// TODO: (the filter id creation process will be done by the programmer on the front end)
//          idea: use authentication to help attribute the filter being added to the user adding it
//                allows user to submit a filter to the database
#[post("/filter")]
pub async fn post_filter(client: web::Data<Client>, body: web::Json<FilterSubmissionBody>) -> HttpResponse {
    let body = body.clone();
    // 1. grab user filter
    let catalog = body.catalog.expect("catalog required");
    let id = body.id.expect("filter id required");
    let permissions = body.permissions.expect("filter permissions required");
    let pipeline = body.pipeline.expect("filter pipeline required");
    
    // Test filter
    
    // 1. create production version of filter
    let filter = build_test_filter(catalog.clone(), id, permissions.clone(), pipeline.clone());
    
    // 2. perform test run to ensure no errors
    match test_filter(client.clone(), catalog.clone(), filter).await {
        Ok(()) => {},
        Err(e) => {
            println!("could not submit filter to database, got error: {}", e);
            return HttpResponse::BadRequest().body(format!("Invalid filter submitted"));
        }
    }

    // save original filter to database
    let filter_collection: Collection<mongodb::bson::Document> = client.database(DB_NAME).collection("filters");
    let database_filter = Filter {
        pipeline: pipeline,
        permissions: permissions,
        catalog: catalog,
        id: id
    };
    let filter_bson = match build_filter_bson(database_filter) {
        Ok(bson) => {
            bson
        },
        Err(e) => {
            println!("unable to create filter bson, got error {}", e);
            return HttpResponse::BadRequest().body("unable to create filter bson");
        }
    };
    let res = filter_collection.insert_one(filter_bson).await;

    HttpResponse::Ok().body("successfully submitted filter to database")
}