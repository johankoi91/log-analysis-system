use futures::future::join_all;
use actix_web::{web, Responder};
use async_tungstenite::tokio::connect_async;
use async_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};
use log::info;
use serde_json::{json, Value};
use tokio::sync::{broadcast, Mutex};
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use std::collections::HashMap;

struct WebSocketClient {
    sender: Arc<Mutex<broadcast::Sender<Value>>>, // Sender 用于发送消息
    receiver: Arc<Mutex<broadcast::Receiver<Value>>>, // Receiver 用于接收消息
}

impl WebSocketClient {
    // WebSocket 客户端连接函数
    async fn connect(url: &str) -> Result<WebSocketClient, String> {
        let connect_timeout = Duration::from_secs(3);
        let ws_url = format!("ws://{}", url);
        match timeout(connect_timeout, connect_async(ws_url)).await {
            Ok(Ok((ws_stream, response))) => {
                println!("Connected to server with status: {}", response.status());

                let (mut write, mut read) = ws_stream.split();
                let send_text = "get_log_source".to_string();
                info!("Sending: {} to log provider ws server", send_text);

                // 创建 broadcast channel
                let (sender, receiver) = broadcast::channel::<Value>(32);

                // 使用 Arc 和 Mutex 来共享 sender 和 receiver
                let sender = Arc::new(Mutex::new(sender));
                let receiver = Arc::new(Mutex::new(receiver));

                // 启动一个任务来处理 WebSocket 的接收
                let sender_clone = Arc::clone(&sender);
                let receiver_clone = Arc::clone(&receiver);
                tokio::spawn(async move {
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                if let Ok(json_data) = serde_json::from_str::<Value>(&text) {
                                    if json_data["cmd"].as_str() == Some("get_log_source") {
                                        // 尝试发送消息，如果锁已经被占用，尝试重试
                                        let _ = sender_clone.lock().await.send(json_data);
                                    }
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

                // 每次请求都发送消息
                write.send(Message::Text(send_text)).await.expect("Failed to send message");
                Ok(WebSocketClient { sender, receiver })
            }
            Ok(Err(err)) => {
                info!("Failed to connect: {}",err);
                Err(format!("Failed to connect: {}", err))
            },
            Err(e) => {
                info!("Connection timeout after waiting for {} ",e);
                Err(format!("Connection timeout after waiting for {} seconds", e))
            },
        }
    }

    // 从 broadcast channel 获取日志源数据
    async fn get_log_source(&self) -> Option<Value> {
        match self.receiver.lock().await.recv().await {
            Ok(value) => Some(value),
            Err(_) => None, // 如果接收失败，返回 None
        }
    }
}


pub async fn discover_node() -> impl Responder {
    let urls = vec![
        "127.0.0.1:9002",
    ];

    // 创建一个 `Mutex` 来确保安全地共享合并结果
    let combined_data = Arc::new(Mutex::new(json!({})));

    // 创建任务列表并发起所有 WebSocket 请求
    let tasks: Vec<_> = urls.into_iter().map(|url| {
        let combined_data = Arc::clone(&combined_data); // 克隆 `Arc` 对象，确保任务间共享

        tokio::spawn(async move {
            let result = match WebSocketClient::connect(url).await {
                Ok(client) => match client.get_log_source().await {
                    Some(valid_data) => Some(valid_data),
                    None => None,
                },
                Err(e) => {
                    info!("WebSocketClient::connect fail: {}",e);
                    None
                }
            };

            // 如果有数据，则将结果合并
            if let Some(data) = result {
                let mut combined_data_lock = combined_data.lock().await;
                combined_data_lock[url] = data; // 将结果放入 `combined_data`
            } else {
                info!("WebSocketClient::connect fail");
            }
        })
    }).collect();

    // 使用 `join_all` 等待所有任务完成
    join_all(tasks).await;

    // 获取并返回合并后的结果
    let combined_data_lock = combined_data.lock().await;
    web::Json(combined_data_lock.clone())  // 返回合并的 JSON 数据
}

// 注册路由
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/discover_node").route(web::get().to(discover_node)));
}
