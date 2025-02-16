use actix_web::{web, Responder};
use async_tungstenite::tokio::connect_async;
use async_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};
use log::info;
use serde_json::{json, Value};
use tokio::time::{timeout, Duration};
use tokio::sync::mpsc;

/*
 {
                                "get_log_source": true,
                                "log_files": [
                                {
                                    "hostname": "host1",
                                    "services": [
                                    {
                                        "service_type": "service1",
                                        "log_files": ["log1", "log2"]
                                    }
                                    ]
                                }
                                ]
                            }
*/

async fn link_ws_server(url: &str, tx: mpsc::Sender<Value>) {
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
            info!("Sending: {} to log provider ws server", send_text);
            write
                .send(Message::Text(send_text))
                .await
                .expect("Failed to send message");

            // 异步接收来自服务器的消息
            tokio::spawn(async move {
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            let result: Result<Value, _> = serde_json::from_str(&text);
                            if let Ok(json_data) = result {
                                if json_data["get_log_source"].as_bool() == Some(true) {
                                    let _ = tx.send(json_data).await;
                                } else {
                                    println!("Received message without valid 'get_log_source' field.");
                                }
                            } else {
                                println!("Failed to parse received message as JSON.");
                            }
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
    // 创建一个通道用于接收 WebSocket 数据
    let (tx, mut rx) = mpsc::channel(1);

    // 启动 WebSocket 连接并传递发送器
    link_ws_server("ws://localhost:9002", tx).await;

    // 等待 WebSocket 数据，并返回作为 HTTP 响应
    match rx.recv().await {
        Some(valid_data) => {
            // 如果收到有效的数据，作为 HTTP 响应返回
            web::Json(valid_data)
        }
        None => {
            // 如果没有收到数据，返回一个默认响应
            web::Json(json!({ "error": "Failed to get log source" }))
        }
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/discover_node").route(web::get().to(discover_node)));
}
