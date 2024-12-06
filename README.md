# README
development environment requirements:
1. active boom mongodb instance
2. postman for query testing

# Api Documentation
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
        "catalog": "ZTF_alerts",
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
    "catalog": "ZTF_alerts",
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

