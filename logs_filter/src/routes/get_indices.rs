use actix_web::{web, Responder};
use serde::Deserialize;
use elasticsearch::{Elasticsearch, IndexParts, cat::CatIndicesParts};
use serde_json::json;

pub async fn get_indices(es: web::Data<Elasticsearch>) -> impl Responder {
    // 获取所有索引
    let response = es.cat().indices(CatIndicesParts::None)
        .format("json")  // 以 JSON 格式返回
        .send()
        .await;

    //  处理响应
    match response {
        Ok(resp) => {
            let body = resp
                .json::<serde_json::Value>()
                .await
                .unwrap_or_else(|_| json!({"error": "Failed to parse response"}));

            // 提取索引名并组合成数组
            let indices: Vec<String> = body
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|entry| entry.get("index").and_then(|index| index.as_str()))
                .map(|index| index.to_string())
                .collect();

           // println!("Indices: {:?}", indices);

            // 返回索引名数组
            web::Json(json!({ "indices": indices }))
        }
        Err(e) => {
            let error_message = format!("Error indexing logs: {}", e);
            web::Json(json!({ "error": error_message }))
        }
    }
}


pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/get_indices").route(web::get().to(get_indices)));
}