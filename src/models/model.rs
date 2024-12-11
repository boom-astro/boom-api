use std::{borrow::Borrow, collections::HashMap, fmt};
use actix_web::{post, web, HttpResponse};
use mongodb::{bson::doc, bson, Client, Collection, Cursor};
use futures::TryStreamExt;

#[derive(serde::Deserialize, Clone)]
pub struct InfoQueryBody {
    pub command: Option<String>,
    pub catalogs: Option<Vec<String>>,
}

#[derive(serde::Deserialize, Clone)]
pub enum Unit {
    Degrees,
    Radians,
    Arcseconds,
    Arcminutes,
}

impl fmt::Debug for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Unit::Degrees => {
                write!(f, "{}", "Degrees")
            }
            Unit::Radians => {
                write!(f, "{}", "Radians")
            }
            Unit::Arcseconds => {
                write!(f, "{}", "Arcseconds")
            }
            Unit::Arcminutes => {
                write!(f, "{}", "Arcminutes")
            }
        }
    }
}

pub struct ConeSearchBody {
    pub radius: Option<f64>,
    pub unit: Option<Unit>,
    pub object_coordinates: Option<HashMap<String, [f64; 2]>>,
    pub catalogs: Option<HashMap<String, [Vec<bson::Document>; 2]>>,
    pub kwargs: Option<QueryKwargs>,
}

// TODO: CHANGE THIS AWEFUL THING BELOW
#[derive(serde::Deserialize, Clone)]
pub struct ObjectCoordinates {
    pub radec: Vec<HashMap<String, [f64; 2]>>, 
    // radec: [f64; 2],
    // ra: f64,
    // dec: f64,
    pub radius: Option<f64>,
    pub unit: Option<Unit>,
}

impl fmt::Debug for ObjectCoordinates {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}, {:?}, {:?}", self.radec, self.radius, self.unit)
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct Query {
    pub object_coordinates: Option<ObjectCoordinates>,
    pub command: Option<String>,
    pub catalog: Option<String>,
    pub filter: Option<mongodb::bson::Document>,
    pub projection: Option<mongodb::bson::Document>,
    pub size: Option<i64>,
}

impl Default for Query {
    fn default() -> Query {
        Query {
            object_coordinates: None,
            command: None,
            catalog: None,
            filter: None,
            projection: None,
            size: None,
        }
    }
}

impl fmt::Debug for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, 
            "{:?},\n{:?},\n{:?},\n{:?},\n{:?},\n{:?}", 
            self.object_coordinates, 
            self.command, 
            self.catalog,
            self.filter,
            self.projection,
            self.size)
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct QueryKwargs {
    pub limit: Option<i64>,
    pub skip: Option<u64>,
    pub sort: Option<mongodb::bson::Document>,
    pub max_time_ms: Option<u64>,
}

impl Default for QueryKwargs {
    fn default() -> QueryKwargs {
        QueryKwargs {
            limit: None,
            skip: None,
            sort: None,
            max_time_ms: None,
        }
    }
}

impl fmt::Debug for QueryKwargs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, 
            "{:?},\n{:?},\n{:?},\n{:?}\n", self.limit, self.skip, self.sort, self.max_time_ms)
    }
}

#[derive(serde::Deserialize)]
pub struct QueryBody {
    // pub query_type: String,
    pub query: Option<Query>,
    pub kwargs: Option<QueryKwargs>,
}
