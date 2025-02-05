mod routes;

use crate::env_logger::Env;
use log::info;
use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use elasticsearch::{indices, Elasticsearch};
use elasticsearch::http::transport::Transport;
use routes::{search, context, unique_services, get_indices};
use env_logger;
use log::error;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志记录
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    info!("Starting Actix Web server...");
    let transport = Transport::single_node("http://localhost:9200").unwrap();
    let client = Elasticsearch::new(transport);
    // 使用 `Data::new` 包装客户端
    let shared_client = web::Data::new(client);

    HttpServer::new(move || {
        App::new()
            .app_data(shared_client.clone())
            // 添加 CORS 配置
            .wrap(
                Cors::default()
                    .allow_any_origin() // 允许所有来源
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![
                        actix_web::http::header::CONTENT_TYPE,
                        actix_web::http::header::AUTHORIZATION,
                    ])
                    .allow_any_header()
                    .max_age(3600), // 缓存预检请求的时间（秒）
            )
            .configure(search::init_routes)
            .configure(context::init_routes)
            .configure(unique_services::init_routes)
            .configure(get_indices::init_routes)
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}