pub mod websocket;
mod modify_filebeat_yaml;

use websocket::{WebSocketServer};
use env_logger;
use env_logger::Env;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    // 启动 WebSocket 服务器
    let mut ws = WebSocketServer::new();

    // 创建一个任务来运行 WebSocketServer
    let server_task = tokio::task::spawn(async move {
        ws.run().await;
    });

    // 如果有其他任务要并发执行，可以在这里创建并等待

    // 等待 server_task 完成
    server_task.await.unwrap();
}
