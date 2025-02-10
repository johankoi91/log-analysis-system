pub mod websocket;
use futures_util::{SinkExt, StreamExt};
use async_std::task;
use websocket::{run};

#[tokio::main]
async fn main() {
    // 启动 WebSocket 服务器
    task::spawn(run());
    tokio::time::sleep(tokio::time::Duration::from_secs(600)).await;

}

// // /// Read YAML configuration file for log directory
// // fn read_log_directory_from_yaml(config_path: &str) -> Option<String> {
// //     let yaml_str = fs::read_to_string(config_path).ok()?;
// //     let yaml_data: serde_yaml::Value = serde_yaml::from_str(&yaml_str).ok()?;
// //     yaml_data["log_directory"].as_str().map(|s| s.to_string())
// // }
// //
// // /// List files in the log directory
// // fn list_log_files(directory: &str) -> Vec<String> {
// //     let path = Path::new(directory);
// //     if path.exists() && path.is_dir() {
// //         fs::read_dir(path)
// //             .unwrap()
// //             .filter_map(|entry| entry.ok())
// //             .filter_map(|entry| entry.file_name().into_string().ok())
// //             .collect()
// //     } else {
// //         vec![]
// //     }
// // }  .collect()
//     } else {
//         vec![]
//     }
// }
