use actix_web::{web, Responder};
use async_tungstenite::tokio::connect_async;
use async_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::time::{timeout, Duration};

async fn link_ws_server(url: &str) {
    // 设置连接超时时间为 5 秒
    let connect_timeout = Duration::from_secs(5);

    // 使用 timeout 包装 connect_async 以处理超时
    match timeout(connect_timeout, connect_async(url)).await {
        Ok(Ok((ws_stream, response))) => {
            println!("Connected to server with status: {}", response.status());

            // 拆分 WebSocket 流，分别处理读写
            let (mut write, mut read) = ws_stream.split();

            // 发送一条文本消息
            let send_text = "get_log_source".to_string();
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
                        }
                        _ => {
                            println!("Received other type of message");
                        }
                    }
                }
            });

            // ws_stream.close(None).await;
            // 等待一段时间后发送关闭消息
            tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
            write
                .send(Message::Close(None))
                .await
                .expect("Failed to send close message");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            println!("WebSocket connection closed.");
        }
        // 处理连接失败的情况
        Ok(Err(err)) => {
            eprintln!("Failed to connect: {}", err);
        }
        // 处理超时的情况
        Err(e) => {
            eprintln!("Connection timeout after waiting for {} seconds", e);
        }
    }
}

pub async fn discover_node() -> impl Responder {
    tokio::spawn(async { link_ws_server("ws://localhost:9002").await });
    web::Json(json!({ "ok": "ok" }))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/discover_node").route(web::get().to(discover_node)));
}
