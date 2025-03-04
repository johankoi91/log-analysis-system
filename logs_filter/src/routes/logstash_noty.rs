use actix_web::{web, Responder};
use serde::Deserialize;
use serde_json::{json, Value};

// #[derive(Deserialize)]
// pub struct SearchRequest {
//     file_name: String,
//     upload_status: String,
// }


pub async fn logstash_noty(
    request: web::Json<Value>,
    body: web::Bytes
) -> impl Responder {
    println!("logstash_noty",);
    // 打印请求体
    println!("Request Body: {:?}", request);

    // 返回 JSON 格式的结果
    web::Json(json!({ "results": "ok" }))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/logstash_noty").route(web::post().to(logstash_noty)));
}
