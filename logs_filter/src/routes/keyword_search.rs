use actix_web::{web, Responder};
use elasticsearch::{Elasticsearch, SearchParts};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct SearchRequest {
    es_index: String, // ES index pattern (e.g., "rtc-logs-*")
    keyword: String,  // Search keyword for the multi_match query
}

pub async fn keyword_search(
    request: web::Json<SearchRequest>,
    es: web::Data<Elasticsearch>,
) -> impl Responder {
    // Construct the query body
    let query = json!({
        "track_total_hits": false,
        "sort": [
            {
                "@timestamp": {
                    "order": "desc",
                    "unmapped_type": "boolean"
                }
            }
        ],
        "fields": [
            {
                "field": "*",
                "include_unmapped": true
            },
            {
                "field": "@timestamp",
                "format": "strict_date_optional_time"
            },
            {
                "field": "event_time",
                "format": "strict_date_optional_time"
            }
        ],
        "size": 500,
        "version": true,
        "script_fields": {},
        "stored_fields": [
            "*"
        ],
        "runtime_mappings": {},
        "_source": false,
        "query": {
            "bool": {
                "must": [],
                "filter": [
                    {
                        "match": {
                            "message": request.keyword,  // 使用请求中的关键字进行 multi_match 查询
                            // "type": "best_fields",
                            // "query": request.keyword, // Using request keyword for multi_match query
                            // "lenient": true
                        }
                    }
                ],
                "should": [],
                "must_not": []
            }
        },
        "highlight": {
            "pre_tags": [
                "@kibana-highlighted-field@"
            ],
            "post_tags": [
                "@/kibana-highlighted-field@"
            ],
            "fields": {
                "*": {}
            },
            "fragment_size": 2147483647
        }
    });

    // Execute the query
    let response = es
        .search(SearchParts::Index(&[&request.es_index])) // Use the index pattern from the request
        .body(query)
        .send()
        .await;

    // Handle the response
    match response {
        Ok(response) => {
            // Assuming the `response` is the JSON response you received
            let body = response.json::<Value>().await.unwrap();

            // Initialize default value for hits
            let default_hits = vec![];

            // Extract relevant data from the body
            let hits = body["hits"]["hits"].as_array().unwrap_or(&default_hits);

            // Prepare the result for JSON response
            let mut result = Vec::new();

            for hit in hits {
                let index = hit["_index"].as_str().unwrap_or_default();
                let file_name = hit["fields"]["log.file.path"]
                    .as_array()
                    .and_then(|arr| arr.get(0))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let message = hit["fields"]["message"]
                    .as_array()
                    .and_then(|arr| arr.get(0))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let timestamp = hit["fields"]["@timestamp"]
                    .as_array()
                    .and_then(|arr| arr.get(0))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let service = hit["fields"]["service"]
                    .as_array()
                    .and_then(|arr| arr.get(0))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let hostname = hit["fields"]["hostname"]
                    .as_array()
                    .and_then(|arr| arr.get(0))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                let entry = json!({
                "_index": index,
                "file_name": file_name,
                "message": message,
                "@timestamp": timestamp,
                "service": service,
                "hostname": hostname
                });
                result.push(entry);
            }
            // Return the results in JSON format
            web::Json(json!({ "results": result }))
        }
        Err(e) => {
            let error_message = format!("Error during search: {}", e);
            web::Json(json!({
                "results": [],
                "error": error_message
            }))
        }
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/keyword_search").route(web::post().to(keyword_search)));
}
