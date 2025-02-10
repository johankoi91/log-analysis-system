use actix_web::{web, Responder};
use serde::Deserialize;
use elasticsearch::{Elasticsearch, IndexParts, cat::CatIndicesParts};
use serde_json::json;
use async_tungstenite::tungstenite::protocol::Message;
use async_tungstenite::tokio::connect_async;
use futures_util::{SinkExt, StreamExt};


async fn link_ws_server() {
    // WebSocket 服务器地址（例如使用 echo 服务）
    let url = "ws://localhost:9002";

    // 建立 WebSocket 连接
    let (ws_stream, response) = connect_async(url)
        .await
        .expect("Failed to connect");
    println!("Connected to server with status: {}", response.status());

    // 拆分 WebSocket 流，分别处理读写
    let (mut write, mut read) = ws_stream.split();

    // 发送一条文本消息
    let send_text = "Hello, WebSocket!".to_string();
    println!("Sending: {}", send_text);
    write
        .send(Message::Text(send_text))
        .await
        .expect("Failed to send message");

    // 异步接收来自服务器的消息
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received text: {}", text);
                }
                Ok(Message::Binary(bin)) => {
                    println!("Received binary: {:?}", bin);
                }
                Ok(Message::Close(frame)) => {
                    println!("Received close message: {:?}", frame);
                    break;
                }
                _ => {
                    println!("Received other type of message");
                }
            }
        }
    });

    // 等待一段时间后发送关闭消息
    tokio::time::sleep(tokio::time::Duration::from_secs(90)).await;
    write
        .send(Message::Close(None))
        .await
        .expect("Failed to send close message");

    // 等待消息发送完毕
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("WebSocket connection closed.");

}


pub async fn discover_node() -> impl Responder {
    link_ws_server().await;

    web::Json(json!({ "ok": "ok" }))

}


pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/discover_node").route(web::get().to(discover_node)));
}