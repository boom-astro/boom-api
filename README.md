# README
development environment requirements:
1. active boom mongodb instance
2. postman for query testing

# Api Documentation
Any endpoints that exist within boom-api must have an entry into this page.

# Query Boom

## Cone Search
**Endpoint**: `POST "/query"`\
**Returns**: HashMap containing object names and resulting documents from it's cone search

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

## Info (W.I.P)
**Endpoint**: `GET "/query/info"`\
**Command**: `string`\
**Enum**: `"catalog_info"`, 
`"index_info"`, `"db_info"`

**Body**:
```
{
    "query": {
        "command": <command enum>
    }
}
```
