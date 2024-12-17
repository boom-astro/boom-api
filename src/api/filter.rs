use mongodb::{action::EstimatedDocumentCount, bson::{doc, document, Document}, Client, Collection};
use actix_web::{post, web, HttpResponse};
use futures::TryStreamExt;
use crate::models::filter_models::*;
use std::error::Error;

const DB_NAME: &str = "boom";

// tests the functionality of a filter by running it on alerts in database
// fn test_filter(built_filter: Filter) -> Result<(), Box<dyn Error>> {
    
// }

struct Filter {
    pub pipeline: Vec<mongodb::bson::Document>,
    pub permissions: Vec<i64>,
    pub catalog: String,
    pub id: i32,
}

// prepends necessary portions to filter to run in database
fn build_filter(
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

// TODO: (the filter id creation process will be done by the programmer on the front end)
//          idea: use authentication to help attribute the filter being added to the user adding it
//                allows user to submit a filter to the database
#[post("/filter")]
pub async fn post_filter(client: web::Data<Client>, body: web::Json<FilterSubmissionBody>) -> HttpResponse {
    let filter_collection: Collection<mongodb::bson::Document> = client.database(DB_NAME).collection("filters");
    let body = body.clone();
    // 1. grab user filter
    let catalog = body.catalog.expect("catalog required");
    let id = body.id.expect("filter id required");
    let permissions = body.permissions.expect("filter permissions required");
    let pipeline = body.pipeline.expect("filter pipeline required");
    
    // Test filter
    
    // 1. prepend portions to filter
    let filter = build_filter(catalog, id, permissions, pipeline);
    println!("{:?}\n{:?}\n{:?}\n{:?}", filter.id, filter.catalog, filter.permissions, filter.pipeline);
    // 2. test it on random alert to ensure no errors
    // match test_filter(filter) {
    //     Ok(()) => {},
    //     Err(e) => {
    //         println!("could not submit filter to database, got error: {}", e);
    //     }
    // }

    // save it to database (non prepended version)
    // todo: write function to send proper packet to database
    // filter_collection.insert_one(doc! {}); // put this in a function

    HttpResponse::Ok().json(doc!{})
}