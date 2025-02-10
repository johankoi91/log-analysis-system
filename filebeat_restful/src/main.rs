pub mod websocket;
use websocket::{WebSocketServer};

#[tokio::main]
async fn main() {
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
