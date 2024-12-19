# README
development environment requirements:
1. active boom mongodb instance
2. postman for query testing

# Api Documentation (new)

# Filtering

## submit filter
**Endpoint**: `POST "/filter`\
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

# Querying

## Info
**Endpoint**: `Get "/query/info"`\
**command_types**: "db_info", "index_info", "catalog_info", "catalog_names"\
**Body**: 
```
{
    "command": <command_type>,
    "catalogs": [catalog_names]
}
```

## Get Object
**Endpoint**: `Get "/alerts/get_object"`\
**Body**:
```
{
    "catalog": <catalog_name>,
    "object_id": <objectId>
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
    },
}
```

## Count Documents
**Endpoint**: `GET "/query/count_documents"`\
**Body:**
```
{
    "query": {
        "catalogs": [<catalog_names>],
        "filter": {},
        "projection": {},

    }
}
```

## Sample
**Endpoint**: `GET "/query/sample"`\
**Body:**
```
{
    "query": {
        "catatlog": <catalog_name>,
        "size": <int>
    }
}
```

## Find
**Endpoint**: `GET "/query/sample"`\
**Body:**
```
{
    "query": {
        
    }
}
```


# Api Documentation (OLD)
Any endpoints that exist within boom-api must have an entry into this page.

# Query Boom

## count documents
**Endpoint**: `POST "/query"`\
**Returns**: Resulting number of documents after running filter on catalog\
**catalog**: String\
**filter**: MQL filter\
**Body**
```
{
  "query_type": "count_documents",
  "query": {
    "catalog": <catalog_name>,
    "filter": { <filter> }
  }
}
```

## Cone Search
**Endpoint**: `POST "/query"`\
**Returns**: HashMap of object names and corresponding cone search results

**Body**:
```
{
    "query_type": "cone_search",
    "query": {
        "object_coordinates": {
            "radius": float,
            "unit": Unit_Enum,
            "radec": [
                {
                    <object_name>: [
                        ra,
                        dec
                    ]
                },
                ...
            ]
        },
        "catalog": <catalog_name>,
        "filter": <mongodb_filter>,
        "projection": <mongodb_projection>
    },
    "kwargs": {
        <KWARGS>
    }
}
```
**Example Body**
```
{
    "query_type": "cone_search",
    "query": {
        "object_coordinates": {
            "radius": 2,
            "unit": "Arcseconds",
            "radec": [
                {"object1": [
                    71.6577756,
                    -10.2263957
                ]},
                {"object2": [
                    82.13523,
                    -12.125
                ]}
            ]
        },
        "catalog": "ZTF",
        "filter": {},
        "projection": {
            "_id": 0,
            "candid": 1,
            "objectId": 1
        }
    },
    "kwargs": {
        "filter_first": false
    }
}
```

## Find 
**Endpoint**: `POST "/query`\
**Body**:
```
{
  "query_type": "find",
  "query": {
    "catalog": "ZTF",
    "filter": {
      "candidate.drb": {
        "$gt": 0.9
      }
    },
    "projection": {
      "_id": 0,
      "candid": 1,
      "candidate.drb": 1
    }
  },
  "kwargs": {
    "limit": 2
  }
}
```

## Info
**Endpoint**: `POST "/query"`\
**Enum**: `"catalog_info"`, 
`"index_info"`, `"db_info"`, `"index_info"`

**Body**:
```
{
    "query": {
        "command": <command enum>
    }
}
```

