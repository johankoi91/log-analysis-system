use actix_web::{web, Responder};
use serde_json::{json, Value};
use elasticsearch::{Elasticsearch, SearchParts};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LogContextRequest {
    start_time: String,  // 时间范围的开始时间
    end_time: String,    // 时间范围的结束时间
    es_index: String,    // Elasticsearch 索引名称
    hostname: String,    // 主机名（必填）
    service: String,     // 服务名（必填）
    basename: String,    // 基准文件名（必填）
}

pub async fn get_log_context(
    request: web::Json<LogContextRequest>,
    es: web::Data<Elasticsearch>,
) -> impl Responder {
    // 构造查询体，合并所有筛选条件，按原始请求方式修改
    let query = json!({
        "query": {
            "bool": {
                "filter": [
                    // {
                    //     "range": {
                    //         "@timestamp": {
                    //             "gte": request.start_time,
                    //             "lte": request.end_time,
                    //             "format": "strict_date_optional_time"  // 时间格式
                    //         }
                    //     }
                    // },
                    {
                        "match_phrase": {
                            "_index": request.es_index  // 索引名筛选
                        }
                    },
                    {
                        "match_phrase": {
                            "service.keyword": request.service  // 服务名筛选
                        }
                    },
                    {
                        "match_phrase": {
                            "hostname.keyword": request.hostname  // 主机名筛选
                        }
                    },
                    {
                        "match_phrase": {
                            "log.file.path.keyword": request.basename  // 基准文件名筛选
                        }
                    }
                ]
            }
        },
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
        "size": 30,
        "version": true,
        "script_fields": {},
        "stored_fields": ["*"],
        "runtime_mappings": {},
        "_source": false,
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
        .search(SearchParts::Index(&[&request.es_index]))  // 使用提供的索引名称
        .body(query)
        .send()
        .await;

    match response {
        Ok(response) => {
            let body = response.json::<Value>().await.unwrap();
            let default_hits = vec![]; // 默认值
            let hits = body["hits"]["hits"].as_array().unwrap_or(&default_hits);

            // 提取上下文数据
            let results: Vec<Value> = hits
                .iter()
                .map(|hit| {
                    let source = &hit["fields"];

                    // 提取字段并处理数组字段
                    let timestamp = source
                        .get("@timestamp")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.get(0)) // 获取数组中的第一个值
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_owned();


                    let message = source
                        .get("message")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.get(0)) // 获取数组中的第一个值
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_owned();

                    let hostname = source
                        .get("hostname")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.get(0)) // 获取数组中的第一个值
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_owned();

                    let service = source
                        .get("service")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
                        .unwrap_or_default();

                    let basename = source
                        .get("basename")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.get(0)) // 获取数组中的第一个值
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_owned();

                    let log_level = source
                        .get("log_level")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.get(0)) // 获取数组中的第一个值
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_owned();

                    // 组合成结构体
                    json!({
            "timestamp": timestamp,
            "message": message,
            "hostname": hostname,
            "service": service,
            "basename": basename,
            "log_level": log_level,
        })
                })
                .collect();

            // 返回日志上下文
            web::Json(json!({ "log_context": results }))
        }
        Err(e) => {
            let error_message = format!("Error during search: {}", e);
            web::Json(json!({
                "log_context": [],
                "error": error_message
            }))
        }
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/get_log_context").route(web::post().to(get_log_context)));
}
