mod routes;
mod config;

use crate::env_logger::Env;
use log::info;
use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use elasticsearch::{Elasticsearch};
use elasticsearch::http::transport::Transport;
use routes::{search, context, unique_services, get_indices, discover_node, logstash_noty,keyword_search};
use env_logger;
use crate::config::read_config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志记录
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let mut es_ip = String::new();
    match read_config() {
        Ok(config) => {
            es_ip  = config.connect_ips.elasticsearch.clone();
        }
        Err(e) => {
            info!("Error reading config: {}", e);
            return Ok(())
        }
    }

    info!("Starting Actix Web server...");
    let transport = Transport::single_node(es_ip.as_str()).unwrap();
    let es_client = Elasticsearch::new(transport);
    let data_es_client = web::Data::new(es_client);

    HttpServer::new(move || {
        App::new()
            .app_data(data_es_client.clone())
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
            .configure(discover_node::init_routes)
            .configure(logstash_noty::init_routes)
            .configure(keyword_search::init_routes)
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}






