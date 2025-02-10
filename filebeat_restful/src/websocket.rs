use async_std::net::{SocketAddr, TcpListener, TcpStream};
use async_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, Result},
};
use futures::{prelude::*};
use log::*;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::{fs, sync::Arc};
use tokio::sync::{Mutex, broadcast};
use std::fs::read_dir;

type SharedClients = Arc<Mutex<Vec<Arc<Mutex<async_tungstenite::WebSocketStream<TcpStream>>>>>>;
type Broadcaster = broadcast::Sender<Message>;

#[derive(Debug, Deserialize)]
#[derive(Clone)]
struct Config {
    log_inputs: Vec<LogInput>,
}

#[derive(Debug, Deserialize)]
#[derive(Clone)]
struct LogInput {
    hostname: String,
    purpose: Vec<ServiceType>,
}

#[derive(Debug, Deserialize)]
#[derive(Clone)]
struct ServiceType {
    service_type: String,
    path: Vec<String>,
}

#[derive(Serialize)]
#[derive(Clone)]
struct LogFiles {
    hostname: String,
    services: Vec<ServiceFiles>,
}

#[derive(Serialize)]
#[derive(Clone)]
struct ServiceFiles {
    service_type: String,
    log_files: Vec<String>,
}

#[derive(Clone)]
pub struct WebSocketServer {
    clients: SharedClients,
    tx: Broadcaster,
    config: Option<Config>, // Store config after loading once
}

impl WebSocketServer {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(2);
        WebSocketServer {
            clients: Arc::new(Mutex::new(Vec::new())),
            tx,
            config: None,
        }
    }

    pub async fn load_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.is_none() {
            let config_path = "/Users/hanxiaoqing/log-searching/filebeat_restful/confg/log.yaml";
            let config_data = fs::read_to_string(config_path)?;
            let config: Config = serde_yaml::from_str(&config_data)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;  // Explicitly convert error
            self.config = Some(config);
        }
        Ok(())
    }


    fn get_files_in_directory(path: &str) -> Vec<String> {
        read_dir(path)
            .ok()
            .into_iter()
            .flat_map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter_map(|entry| entry.file_name().to_str().map(String::from))
                    .collect::<Vec<String>>()
            })
            .collect()
    }
    pub async fn run(&mut self) {
        let addr = "127.0.0.1:9002";
        let listener = TcpListener::bind(&addr).await.expect("Can't listen");
        info!("Listening on: {}", addr);

        // 先加载配置
        if let Err(e) = self.load_config().await {
            error!("Error loading config: {}", e);
            return;
        }

        // 克隆 `self`，然后用 Arc<Mutex<Self>> 包装
        let server_clone = self.clone();
        let server_arc = Arc::new(Mutex::new(server_clone));

        // 克隆 tx 以便传递给异步任务
        let tx_clone = self.tx.clone();

        // 使用 async move，将 `self` 移入异步任务中
        tokio::spawn({
            let config_clone = self.config.clone();
            async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
                    // tokio::task::sleep(std::time::Duration::from_secs(4)).await;

                    if let Some(config) = &config_clone {
                        let log_files: Vec<LogFiles> = config.log_inputs.iter().map(|input| {
                            let services = input.purpose.iter().map(|service| {
                                ServiceFiles {
                                    service_type: service.service_type.clone(),
                                    log_files: service.path.iter().flat_map(|path| {
                                        WebSocketServer::get_files_in_directory(path)
                                    }).collect(),
                                }
                            }).collect();

                            LogFiles {
                                hostname: input.hostname.clone(),
                                services,
                            }
                        }).collect();

                        let json = serde_json::to_vec(&log_files).expect("Failed to serialize to JSON");
                        let _ = tx_clone.send(Message::Binary(json));
                    }
                }
            }
        });

        while let Ok((stream, _)) = listener.accept().await {
            let peer = stream.peer_addr().expect("Connected streams should have a peer address");
            println!("Peer address: {}", peer);

            let clients_clone = self.clients.clone();
            let tx_clone = self.tx.clone();
            // 使用 Arc<Mutex<Self>> 共享 `self`
            let server_arc_clone = Arc::clone(&server_arc);
            tokio::spawn({
                let server_arc_clone = Arc::clone(&server_arc_clone);
                async move {
                    let server = server_arc_clone.lock().await;
                    server.accept_connection(peer, stream, clients_clone, tx_clone).await;
                }
            });
        }
    }



    pub async fn accept_connection(
        &self,
        peer: SocketAddr,
        stream: TcpStream,
        clients: SharedClients,
        tx: Broadcaster,
    ) {
        if let Err(e) = self.handle_connection(peer, stream, clients, tx).await {
            match e {
                Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                err => error!("Error processing connection: {}", err),
            }
        }
    }

    pub async fn handle_connection(
        &self,
        peer: SocketAddr,
        stream: TcpStream,
        clients: SharedClients,
        tx: Broadcaster,
    ) -> Result<()> {
        let ws_stream = accept_async(stream).await.expect("Failed to accept");
        let ws_stream = Arc::new(Mutex::new(ws_stream));
        info!("New WebSocket connection: {}", peer);

        // 监听广播消息
        let mut rx = tx.subscribe();

        // 存储客户端连接
        {
            let mut clients_lock = clients.lock().await;
            clients_lock.push(Arc::clone(&ws_stream));
        }

        loop {
            tokio::select! {
                Some(msg) = async {
                    let mut ws_guard = ws_stream.lock().await; // 使用异步 Mutex
                    ws_guard.next().await
                } => {
                    let msg = msg?;
                    if msg.is_text() || msg.is_binary() {
                        // 处理消息并广播
                        let _ = tx.send(msg.clone());
                    }
                }
                Ok(msg) = rx.recv() => {
                    // 监听广播消息并转发给所有客户端
                    self.broadcast_message(&clients, msg).await;
                }
            }
        }
    }

    // 广播消息给所有客户端
    async fn broadcast_message(&self, clients: &SharedClients, msg: Message) {
        let clients_lock = clients.lock().await;
        for client in clients_lock.iter() {
            let mut client = client.lock().await;
            let _ = client.send(msg.clone()).await;
        }
    }
}
