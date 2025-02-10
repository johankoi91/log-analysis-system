use async_std::net::{SocketAddr, TcpListener, TcpStream};
use async_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, Result},
};
use futures::{prelude::*, SinkExt};
use log::*;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

type SharedClients = Arc<Mutex<Vec<Arc<Mutex<async_tungstenite::WebSocketStream<TcpStream>>>>>>;
type Broadcaster = broadcast::Sender<Message>;

async fn accept_connection(
    peer: SocketAddr,
    stream: TcpStream,
    clients: SharedClients,
    tx: Broadcaster,
) {
    if let Err(e) = handle_connection(peer, stream, clients, tx).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(
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
                broadcast_message(&clients, msg).await;
            }
        }
    }
}

// 广播消息给所有客户端
async fn broadcast_message(clients: &SharedClients, msg: Message) {
    let clients_lock = clients.lock().await;
    for client in clients_lock.iter() {
        let mut client = client.lock().await;
        let _ = client.send(msg.clone()).await;
    }
}

pub async fn run() {
    let addr = "127.0.0.1:9002";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);

    // 共享 WebSocket 连接集合
    let clients: SharedClients = Arc::new(Mutex::new(Vec::new()));

    // 创建广播通道（buffer 100 条）
    let (tx, _) = broadcast::channel(100);

    // 允许服务器在任何时候发送自定义消息
    let tx_clone = tx.clone();
    async_std::task::spawn(async move {
        loop {
            async_std::task::sleep(std::time::Duration::from_secs(10)).await;
            let _ = tx_clone.send(Message::Text("服务器通知：请检查更新".to_string()));
        }
    });

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("Connected streams should have a peer address");
        println!("Peer address: {}", peer);

        let clients_clone = clients.clone();
        let tx_clone = tx.clone();
        async_std::task::spawn(accept_connection(peer, stream, clients_clone, tx_clone));
    }
}
