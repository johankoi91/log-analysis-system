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
        info!("websocket service is listening on: {}", addr);

        // 先加载配置
        if let Err(e) = self.load_config().await {
            error!("Error loading config: {}", e);
            return;
        }

        let tx_clone = self.tx.clone();

        tokio::spawn({
            async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
                    let _ = tx_clone.send(Message::text("broadcast msg"));
                }
            }
        });

        let server_clone = self.clone();
        let server_arc = Arc::new(Mutex::new(server_clone));

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
        let ws_stream = Arc::new(Mutex::new(ws_stream)); // Wrap WebSocketStream inside a Mutex

        info!("New WebSocket connection: {}", peer);

        let mut clients_lock = clients.lock().await;
        clients_lock.push(ws_stream.clone()); // Use Arc::clone to share the reference

        // Listen for broadcast messages
        let mut rx = tx.subscribe();

        loop {
            tokio::select! {
            Some(msg) = async {
                let mut ws_guard = ws_stream.lock().await; // Lock the WebSocketStream for mutable access
                ws_guard.next().await
            } => {
                let msg = msg?;
                self.handle_client_message(msg, peer, &tx, &ws_stream).await;
            }
            Ok(msg) = rx.recv() => {
                // Listen for broadcast messages and forward them to all clients
                self.broadcast_message(&clients, msg).await;
            }
        }
        }
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
                    let log_files: Vec<LogFiles> = config.log_inputs.iter().map(|input| {
                        let services = input.purpose.iter().map(|service| {
                            ServiceFiles {
                                service_type: service.service_type.clone(),
                                log_files: service.path.iter().flat_map(|path| {
                                    WebSocketServer::get_files_in_directory(path)
                                }).collect(),
                            }
                        }).collect();
                        LogFiles { hostname: input.hostname.clone(), services, }
                    }).collect();

                    let response_json = serde_json::to_string(&log_files).expect("Failed to serialize to JSON");
                    let response_msg = Message::Text(response_json);
                    // 发送消息给当前客户端
                    let mut client_ws_guard = client_ws.lock().await;
                    client_ws_guard.send(response_msg).await;
                    return;
                }
            }
        } else if msg.is_binary() {
            info!("Received binary message from {}", peer);
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
