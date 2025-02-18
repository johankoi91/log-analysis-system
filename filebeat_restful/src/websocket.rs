use async_std::net::{SocketAddr, TcpListener, TcpStream};
use async_tungstenite::{
    accept_async,
    WebSocketStream,
    tungstenite::{Error, Message, Result},
};
use futures::prelude::*;
use log::*;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::read_dir;
use std::{fs, sync::Arc};
use std::ptr::read;
use tokio::sync::{broadcast, Mutex};

type SharedClients = Arc<Mutex<Vec<Arc<Mutex<WebSocketStream<TcpStream>>>>>>;
type Broadcaster = broadcast::Sender<Message>;

#[derive(Debug, Deserialize, Clone)]
struct Config {
    log_inputs: Vec<LogInput>,
}

#[derive(Debug, Deserialize, Clone)]
struct LogInput {
    hostname: String,
    purpose: Vec<ServiceType>,
}

#[derive(Debug, Deserialize, Clone)]
struct ServiceType {
    service_type: String,
    path: Vec<String>,
}

#[derive(Serialize, Clone)]
struct LogFiles {
    hostname: String,
    services: Vec<ServiceFiles>,
}

#[derive(Serialize, Clone)]
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
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?; // Explicitly convert error
            self.config = Some(config);
        }
        Ok(())
    }

    fn get_files_in_directory(path: &str) -> Vec<String> {
        fs::read_dir(path)
            .and_then(|entries| {
                entries
                    .map(|entry| entry.map(|e| e.file_name().into_string().unwrap_or_default()))
                    .collect()
            })
            .unwrap_or_default() // 如果出错，返回空 Vec
    }

    pub async fn run(&mut self) {
        let addr = "127.0.0.1:9002";
        let listener = TcpListener::bind(&addr).await.expect("Can't listen");
        info!("WebSocket service is listening on: {}", addr);

        // 先加载配置
        if let Err(e) = self.load_config().await {
            error!("Error loading config: {}", e);
            return;
        }

        let server_arc = Arc::new(Mutex::new(self.clone()));

        while let Ok((stream, _)) = listener.accept().await {
            let peer = stream.peer_addr().expect("Connected streams should have a peer address");
            info!("Connected a peer address: {}", peer);

            let clients_clone = self.clients.clone();
            let tx_clone = self.tx.clone();

            // 使用 Arc<Mutex<Self>> 共享 `self`
            let server_arc_clone = Arc::clone(&server_arc);

            tokio::spawn({
                async move {
                    let server = server_arc_clone.lock().await;
                    info!("before call accept_connection");
                    server.accept_connection(peer, stream, clients_clone, tx_clone).await;
                    info!("after call accept_connection");
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
        info!("Starting accept_connection for peer: {}", peer);  // 打印开始信息
        if let Err(e) = self.handle_connection(peer, stream, clients, tx).await {
            match e {
                Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                err => info!("Error processing connection: {}", err),
            }
        }
        info!("Exiting accept_connection for peer: {}", peer);  // 打印结束信息
    }

    pub async fn handle_connection(
        &self,
        peer: SocketAddr,
        stream: TcpStream,
        clients: SharedClients,
        tx: Broadcaster,
    ) -> Result<()> {
        let ws_stream = accept_async(stream).await.expect("Failed to accept");
        let ws_stream = Arc::new(Mutex::new(ws_stream)); // Wrap WebSocketStream inside a Mutex

        info!("New WebSocket connection: {}", peer);

        let mut clients_lock = clients.lock().await;
        clients_lock.push(ws_stream.clone()); // Use Arc::clone to share the reference

        // Listen for broadcast messages
        let mut rx = tx.subscribe();

        info!(" begin handle_client_message");
        loop {
            tokio::select! {
            Some(msg) = async {
                let mut ws_guard = ws_stream.lock().await; // Lock the WebSocketStream for mutable access
                ws_guard.next().await
            } => {

            match msg {
                            Ok(msg) => {
                                self.handle_client_message(msg, peer, &tx, &ws_stream).await;
                                return Ok(());
                            },
                            Err(e) => {
                                info!("Error processing message: {}", e);
                                return Err(e);
                             }
                        };

            }
            // Ok(msg) = rx.recv() => {
            //     // Listen for broadct messages and forward them to all clients
            //         if let Ok(msg) = rx.recv().await {  // 使用 `try_recv` 避免阻塞
            //             self.broadcast_message(&clients, msg).await;
            //         }
            //     }
            }
        }
        info!("after tokio::select handle_client_message");
        // 连接断开后，移除 WebSocket 客户端
        info!("Client {} disconnected", peer);

        // let mut clients_lock = clients.lock().await;
        // clients_lock.close();
        // clients_lock.retain(|client| !Arc::ptr_eq(client, &ws_stream));

        // Ok(())
    }

    async fn handle_client_message(
        &self,
        msg: Message,
        peer: SocketAddr,
        tx: &Broadcaster,
        client_ws: &Arc<Mutex<async_tungstenite::WebSocketStream<TcpStream>>>,
    ) {
        if msg.is_text() {
            let text = msg.to_text().unwrap();
            info!("Received text message from {}: {}", peer, text);

            // 判断是否是 "get_log_source"
            if text == "get_log_source" {
                info!("Processing get_log_source from {} for get log files", peer);
                if let Some(config) = &self.config {
                    let log_files: Vec<LogFiles> = config
                        .log_inputs
                        .iter()
                        .map(|input| {
                            let services = input
                                .purpose
                                .iter()
                                .map(|service| ServiceFiles {
                                    service_type: service.service_type.clone(),
                                    log_files: service
                                        .path
                                        .iter()
                                        .flat_map(|path| {
                                            WebSocketServer::get_files_in_directory(path)
                                        })
                                        .collect(),
                                })
                                .collect();
                            LogFiles {
                                hostname: input.hostname.clone(),
                                services,
                            }
                        })
                        .collect();

                    let response_log_files =
                        serde_json::to_string(&log_files).expect("Failed to serialize to JSON");
                    let  response_json = format!(
                        r#"{{"get_log_source": true, "log_files": {}}}"#,
                        response_log_files
                    );
                    info!("response_log_files: {} ", response_json);
                    let response_msg = Message::Text(response_json);
                    let mut client_ws_guard = client_ws.lock().await;
                    client_ws_guard.send(response_msg).await;

                    return;
                }
            }
        } else if msg.is_binary() {
            info!("Received binary message from {}", peer);
        } else if msg.is_close() {
            info!("Received is_close");
            let mut client_ws_guard = client_ws.lock().await;
            client_ws_guard.close(None).await;
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
