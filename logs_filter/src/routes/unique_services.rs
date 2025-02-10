use actix_web::{web, Responder};
use serde::Deserialize;
use elasticsearch::{Elasticsearch, SearchParts};
use serde_json::json;

#[derive(Deserialize)]
pub struct SearchParams {
    index_pattern: String, // 用于指定索引的模式，比如 rtc-logs-*
    field: String,         // 用于指定聚合字段，比如 hostname.keyword
}

pub async fn get_unique_services(
    es: web::Data<Elasticsearch>,
    params: web::Query<SearchParams>,
) -> impl Responder {
    // 使用查询参数构造查询体
    let query = json!({
        "size": 0,
        "aggs": {
            "unique_services": {
                "terms": {
                    "field": params.field,
                    "size": 10000
                }
            }
        }
    });

    // 执行查询
    let response = es.search(SearchParts::Index(&[&params.index_pattern]))
        .body(query)
        .send()
        .await;

    // 处理响应
    match response {
        Ok(resp) => {
            let body = resp
                .json::<serde_json::Value>()
                .await
                .unwrap_or_else(|_| json!({"error": "Failed to parse response"}));
            println!("Raw response body: {:?}", body);

            // 提取聚合结果中的服务
            let unique_services = body
                .get("aggregations")                       // 获取 aggregations 部分
                .and_then(|aggs| aggs.get("unique_services"))  // 获取 unique_services 部分
                .and_then(|unique_services| unique_services.get("buckets")) // 获取 buckets 部分
                .and_then(|buckets| buckets.as_array())      // 转换成数组
                .unwrap_or(&vec![])                          // 如果为 None 则返回空的 Vec
                .iter()
                .filter_map(|bucket| {
                    // 从每个 bucket 中获取 key 字段（即 hostname.keyword）
                    bucket.get("key").and_then(|key| key.as_str()).map(|key| key.to_string())
                })
                .collect::<Vec<String>>();                  // 收集所有 key 到 Vec

            println!("Unique services: {:?}", unique_services);

            // 返回唯一服务（hostname.keyword）列表
            web::Json(json!({ "unique_services": unique_services }))
        }
        Err(e) => {
            let error_message = format!("Error indexing logs: {}", e);
            web::Json(json!({ "error": error_message }))
        }
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/indices").route(web::get().to(get_unique_services)));
}