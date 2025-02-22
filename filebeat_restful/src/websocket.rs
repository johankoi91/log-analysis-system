use crate::modify_filebeat_yaml::{modify_yaml,modify_yaml_dynamic};
use crate::system_cmd;
use async_std::net::{SocketAddr, TcpListener, TcpStream};
use async_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, Result},
    WebSocketStream,
};
use futures::prelude::*;
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_yaml;
use std::fs::read_dir;
use std::ptr::read;
use std::{fs, sync::Arc};
use tokio::sync::{broadcast, Mutex};
use std::env;

type SharedClients = Arc<Mutex<Vec<Arc<Mutex<WebSocketStream<TcpStream>>>>>>;
type Broadcaster = broadcast::Sender<Message>;

#[derive(Debug, Deserialize, Clone)]
struct Config {
    log_inputs: Vec<ServiceType>,
}

#[derive(Debug, Deserialize, Clone)]
struct ServiceType {
    service_type: String,
    path: Vec<String>,
}

#[derive(Serialize, Clone)]
struct ServiceFiles {
    service_type: String,
    dir: String,
    log_files: Vec<String>,
}

#[derive(Serialize)]
struct Response {
    cmd: String,
    services: Vec<ServiceFiles>, // 或者替换成你期望的具体类型
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
            let file_path = env::var("LOG_FILE_PATH").unwrap_or_else(|_| "/Users/hanxiaoqing/log-searching/filebeat_restful/config/log.yaml".to_string());
            let config_data = fs::read_to_string(file_path)?;
            info!("load_config: {}", config_data);
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
        let addr = "0.0.0.0:9002";
        let listener = TcpListener::bind(&addr).await.expect("Can't listen");
        info!("WebSocket service is listening on: {}", addr);

        // 先加载配置
        if let Err(e) = self.load_config().await {
            info!("Error loading config: {}", e);
            return;
        }

        let server_arc = Arc::new(Mutex::new(self.clone()));

        while let Ok((stream, _)) = listener.accept().await {
            let peer = stream
                .peer_addr()
                .expect("Connected streams should have a peer address");
            info!("Connected a peer address: {}", peer);

            let clients_clone = self.clients.clone();
            let tx_clone = self.tx.clone();

            let server_arc_clone = Arc::clone(&server_arc);

            tokio::spawn({
                async move {
                    let server = server_arc_clone.lock().await;
                    info!("before call accept_connection");
                    server
                        .accept_connection(peer, stream, clients_clone, tx_clone)
                        .await;
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
        info!("Starting accept_connection for peer: {}", peer); // 打印开始信息
        if let Err(e) = self.handle_connection(peer, stream, clients, tx).await {
            match e {
                Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                err => info!("Error processing connection: {}", err),
            }
        }
        info!("Exiting accept_connection for peer: {}", peer); // 打印结束信息
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
                        }};
                }
            }
        }
        info!("after tokio::select handle_client_message");
    }

    async fn handle_client_message(
        &self,
        msg: Message,
        peer: SocketAddr,
        tx: &Broadcaster,
        client_ws: &Arc<Mutex<WebSocketStream<TcpStream>>>,
    ) {
        if msg.is_text() {
            let text = msg.to_text().unwrap();
            if text == "get_log_source" {
                info!("Received cmd：{} from {} for get log files", text, peer);
                if let Some(config) = &self.config {
                    let log_files: Vec<ServiceFiles> = config
                        .log_inputs
                        .iter()
                        .flat_map(|inputs_kv| {
                            inputs_kv.path.iter().map(|path| ServiceFiles {
                                service_type: inputs_kv.service_type.clone(),
                                dir: path.clone(),
                                log_files: WebSocketServer::get_files_in_directory(path),
                            })
                        })
                        .collect();

                    let response = Response {
                        cmd: text.to_string(),
                        services: log_files,
                    };

                    let response_json =
                        serde_json::to_string(&response).expect("Failed to serialize to JSON");
                    info!("response_log_files: {} ", response_json);
                    let response_msg = Message::Text(response_json);
                    let mut client_ws_guard = client_ws.lock().await;
                    let _ = client_ws_guard.send(response_msg).await;
                    return;
                }
            }

            if let Ok(json_data) = serde_json::from_str::<Value>(&text) {
                if json_data["cmd"].as_str() == Some("firebase_upload") {
                    let file_path = env::var("FILEBEAT_CONFIG_LOG_PATH").unwrap_or_else(|_| "/Users/hanxiaoqing/log-searching/filebeat_restful/filebeat/inputs.d/log.yml".to_string());
                    let new_paths = vec![json_data["upload_file"].as_str().unwrap_or_default().to_string()];
                    let new_hostname = json_data["hostname"].as_str().unwrap_or_default().to_string();
                    let new_service = json_data["service"].as_str().unwrap_or_default().to_string();
                    info!("Received cmd: firebase_upload from {},need change file_path:{}, new_paths:{}, new_service:{}, new_hostname:{}", peer,file_path,new_paths[0],new_service,new_hostname);
                    modify_yaml_dynamic(file_path.as_str(), new_paths, new_service, new_hostname);
                    let response_msg = Message::Text("firebase_upload ing".to_string());
                    let mut client_ws_guard = client_ws.lock().await;
                    client_ws_guard.send(response_msg).await;

                    let filebeat_config_path = env::var("FILEBEAT_CONFIG_MAIN_PATH")
                        .unwrap_or_else(|_| "/Users/hanxiaoqing/log-searching/filebeat_restful/filebeat/filebeat.yml".to_string());
                    info!("filebeat_config_path: {}", filebeat_config_path);
                    if env::var("FILEBEAT_CONFIG_MAIN_PATH").is_ok() {
                        match system_cmd::start_filebeat(filebeat_config_path.as_str()).await {
                            Ok(()) => {
                                info!("Filebeat started successfully with config: {}", filebeat_config_path);
                                return;
                            }
                            Err(e) => {
                                info!("start_filebeat Error: {}", e);
                                return;
                            }
                        }
                    } else {
                        match system_cmd::get_and_restart_container("filebeat").await {
                            Ok(()) => {
                                info!("Filebeat container restarted successfully.");
                                return;
                            }
                            Err(e) => {
                                info!("Filebeat container restarted Error: {}", e);
                                return;
                            }
                        }
                    }
                }
                return;
            }
        } else if msg.is_binary() {
            info!("Received binary message from {}", peer);
        } else if msg.is_close() {
            info!("Received is_close");
            let mut client_ws_guard = client_ws.lock().await;
            client_ws_guard.close(None).await;
        }
    }
}
