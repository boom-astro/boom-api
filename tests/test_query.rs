use boom_api::api::query;
#[cfg(test)]

use boom_api::{
    models::{query_models::{QueryBody, QueryKwargs, Query, Unit,}},
    api::{query::build_options}
};
use mongodb::{bson::{doc, Document, Array}, options::FindOptions};

// checks if two FindOptions structs have equal member values.
// only checks members which are accessed by boom_api::api::query::build_options
pub fn check_find_options_equal(a: FindOptions, b: FindOptions) -> bool {
    assert_eq!(a.limit, b.limit);
    assert_eq!(a.skip, b.skip);
    assert_eq!(a.sort, b.sort);
    assert_eq!(a.max_time, b.max_time);
    return true;
}

// UNIT TESTS

#[actix_rt::test]
async fn test_build_options() {

    let test_projection = Some(doc! {
        "$project": {
            "objectId": 1, 
            "candid": 1, 
            "candidate": 1,
        }
    });

    let test_kwargs = QueryKwargs {
        limit: Some(5),
        ..Default::default()
    };

    // test defaults
    let default_options_test = build_options(None, QueryKwargs {..Default::default()}).await;
    let default_options = mongodb::options::FindOptions::default();
    assert!(check_find_options_equal(default_options, default_options_test));

    // test sample projection and default kwargs
    let proj_options_test = build_options(test_projection.clone(), QueryKwargs {..Default::default()}).await;
    let mut proj_options = mongodb::options::FindOptions::default();
    proj_options.projection = test_projection.clone();
    assert!(check_find_options_equal(proj_options_test, proj_options));

    // test sample projection with sample kwargs
    let full_options_test = build_options(test_projection.clone(), test_kwargs.clone()).await;
    let mut full_options = mongodb::options::FindOptions::default();
    full_options.projection = test_projection.clone();
    full_options.limit = test_kwargs.limit;
    assert!(check_find_options_equal(full_options, full_options_test));
}


// TODO: finish this function
// issues:
// trying to do filter_insert
// 1) look into how Vec of documents can be viewed as a single filter
// 2) if it doesn't work, just use a single doc! call instead of vec! inside of doc! call.
#[actix_rt::test]
async fn test_build_cone_search_filter() {
    let radec = (91.0, 188.0);
    let unit = Unit::Degrees;
    let radius: f64 = 16.0;

    let filter= doc! {
        "$project": {
            "cutoutScience": 0, 
            "cutoutDifference": 0, 
            "cutoutTemplate": 0,
            "publisher": 0, 
            "schemavsn": 0
        }
    };
    let radius_a = radius.to_radians();
    let ra = radec.0 - 180.0;
    let dec = radec.1;
    let center_sphere = doc! {
        "$centerSphere": [[ra, dec], radius_a]
    };
    let geo_within = doc! {
        "$geoWithin": center_sphere
    };
    
    let mut filter_a = filter.clone();
    filter_a.insert("coordinates.radec_geojson", geo_within);
    
    let built_filter = query::build_cone_search_filter(filter, radec, radius, unit).await;
    
    assert_eq!(filter_a, built_filter);

}

// TODO: test get_info functionality

#[actix_rt::test]
async fn test_get_info() {

}