use actix_web::{web, Responder};
use elasticsearch::{Elasticsearch, SearchParts};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct SearchRequest {
    es_index: String,
    keyword: String,   // 用于 multi_match 查询的关键字
    start_time: String, // 时间范围的开始时间
    end_time: String,   // 时间范围的结束时间
    hostname: String,   // 主机名
    service: String,    // 服务名
    basename: String,   // 文件名
}


pub async fn search_logs(
    request: web::Json<SearchRequest>,
    es: web::Data<Elasticsearch>,
) -> impl Responder {
    // 构造查询体
    let query = json!({
        "track_total_hits": false,
        "sort": [
            {
                "@timestamp": {
                    "order": "asc",
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
            }
        ],
        "size": 500,
        "query": {
            "bool": {
                "filter": [
                    {
                        "match_phrase": {
                            "message": request.keyword,  // 使用请求中的关键字进行 multi_match 查询
                        }
                    },
                    {
                        "range": {
                            "@timestamp": {
                                "format": "strict_date_optional_time",
                                "gte": request.start_time,
                                "lte": request.end_time
                            }
                        }
                    },
                    {
                        "bool": {
                            "must": [],
                            "filter": [
                                {
                                    "match_phrase": {
                                        "hostname": request.hostname
                                    }
                                },
                                {
                                    "match_phrase": {
                                        "service": request.service
                                    }
                                },
                                {
                                    "match_phrase": {
                                        "basename": request.basename
                                    }
                                }
                            ],
                            "should": [],
                            "must_not": []
                        }
                    }
                ]
            }
        },
        "highlight": {
            "pre_tags": ["@kibana-highlighted-field@"],
            "post_tags": ["@/kibana-highlighted-field@"],
            "fields": {
                "*": {}
            },
            "fragment_size": 2147483647
        }
    });

    // 执行查询
    let response = es
        .search(SearchParts::Index(&[&request.es_index])) // 替换为目标索引
        .body(query)
        .send()
        .await;

    // 处理响应
    match response {
        Ok(response) => {
            let body = response.json::<Value>().await.unwrap();
            let default_hits = vec![]; // 默认值
            let hits = body["hits"]["hits"].as_array().unwrap_or(&default_hits);

            let results: Vec<Value> = hits
                .iter()
                .map(|hit| {
                    let source = &hit["_source"];
                    let hostname = source
                        .get("hostname")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned();
                    let basename = source
                        .get("basename")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned();
                    let message = source
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned();
                    let timestamp = source
                        .get("@timestamp")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned();
                    let log_level = source
                        .get("log_level")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned();

                    json!({
                        "hostname": hostname,
                        "basename": basename,
                        "message": message,
                        "timestamp": timestamp,
                        "log_level": log_level
                    })
                })
                .collect();

            // 返回 JSON 格式的结果
            web::Json(json!({ "results": results }))
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
    cfg.service(web::resource("/search").route(web::post().to(search_logs)));
}
