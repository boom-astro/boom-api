# README
development environment requirements:
1. active boom mongodb instance
2. postman (or some way of making api calls) for querying

# Api Documentation

# Filtering (W.I.P.)

## Submit a new filter
**Endpoint**: `POST "/filter"`\
**Body**:
```
{
    "pipeline": aggregate pipeline (array of bson documents),
    "catalog": catalog name (string),
    "permissions": allowed permissions,
    "id": filter id (i32)
}
```

**Example Body**:
```
{
    "pipeline": 
    [   
        {
            "$project": {
                "cutoutScience": 0, 
                "cutoutDifference": 0, 
                "cutoutTemplate": 0, 
                "publisher": 0, 
                "schemavsn": 0
            }
        }, 
        {
            "$lookup": {
                "from": "alerts_aux", 
                "localField": "objectId", 
                "foreignField": "_id", 
                "as": "aux"
            }
        }, 
        {
            "$project": {
                "objectId": 1, 
                "candid": 1, 
                "candidate": 1, 
                "classifications": 1, 
                "coordinates": 1, 
                "prv_candidates": {
                    "$arrayElemAt": [
                        "$aux.prv_candidates", 
                        0
                    ]
                }, 
                "cross_matches": {
                    "$arrayElemAt": [
                        "$aux.cross_matches", 
                        0
                    ]
                }
            }
        }, 
        {
            "$match": {
                "candidate.drb": {
                    "$gt": 0.5
                }, 
                "candidate.ndethist": {
                    "$gt": 1.0
                }, 
                "candidate.magpsf": {
                    "$lte": 18.5
                }
            }
        }
    ],
    "catalog": "ZTF",
    "permissions": [1],
    "id": -3
}
```

## Add new pipeline version to existing filter
**Endpoint**: `PATCH "/filter/filter_id"`\
**Body**:
```
{
    "pipeline": aggregate pipeline (array of bson documents)
}
```

# Querying

## Info
**Endpoint**: `Get "/query/info"`\
**command_types**: "db_info", "index_info", "catalog_info", "catalog_names"\
**catalog_names**: Array Strings. e.g., `["ZTF_alerts",...]` (not required for db_info, catalog_names)\
**Body**: 
```
{
    "command": <command_type>,
    "catalogs": [catalog_names]
}
```

## Get Object
**Endpoint**: `Get "/alerts/get_object"`\
**catalog_name**: String. e.g., "ZTF_alerts"\
**Body**:
```
{
    "catalog": <catalog_name>,
    "object_id": <objectId>
}
```
**Example Body**:
```
{
    "catalog": "ZTF",
    "object_id": "ZTF18aajpnun"
}
```

## Cone Search
**Endpoint**: `Get "/query/cone_search"`\
**Unit**: "Arcseconds", "Arcminutes", "Degrees", "Radians"\
**Body**:
```
{
    "radius": <float>,
    "unit": <Unit>,
    "object_coordinates": {
        <object_name>: [
            <ra>, <dec>
        ],
        <object2_name>: [
            <ra>, <dec>
        ]
    },
    "catalog": {
        "catalog_name": <catalog_name>,
        "filter": <bson>,
        "projection": <bson>
    },
    "kwargs": {<kwargs>}
}
```

**Example Body** (should return at least an object called `NGC 5162`):
```
{
    "radius": 1,
    "unit": "Arcseconds",
    "object_coordinates": {
        "object1": [
            202.366276, 11.006276
        ]
    },
    "catalog": {
        "catalog_name": "NED",
        "filter": {},
        "projection": {}
    }
}
```

## Count Documents
**Endpoint**: `GET "/query/count_documents"`\
**catalog_name**: String. e.g., "ZTF_alerts"\
**Body:**
```
{
    "query": {
        "catalog": <catalog_name>,
        "filter": {},
        "projection": {},
    }
}
```

## Sample
**Endpoint**: `GET "/query/sample"`\
**catalog_name**: String. e.g., "ZTF_alerts"\
**Body:**
```
{
    "query": {
        "catalog": <catalog_name>,
        "size": <int>
    }
}
```

## Find
**Endpoint**: `GET "/query/find"`\
**catalog_name**: String. e.g., "ZTF_alerts"\
**Body:**
```
{
    "query": {
        "catalog": <catalog_name>,
        "filter": <bson filter (aggregate pipeline)>
    },
    "kwargs": {<kwargs>}
}
```
**Example Body**:
```
{
    "query": {
        "catalog": "ZTF_alerts",
        "filter": {}
    },
    "kwargs": {
        "limit": 1
    }
}
```
