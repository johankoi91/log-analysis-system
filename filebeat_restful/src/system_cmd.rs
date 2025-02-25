use std::str;
use log::info;
use tokio::io::{self, AsyncWriteExt};
use tokio::process::{Command};
use std::process::Stdio; // 从标准库导入 Stdio

pub(crate) async fn start_filebeat(filebeat_config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let filebeat_output = Command::new("/usr/share/filebeat/filebeat")
        .arg("-e")
        .arg("-c")
        .arg(filebeat_config_path)
        .output()
        .await;

    match filebeat_output {
        Ok(output) => {
            if !output.status.success() {
                // 打印出标准输出和标准错误信息
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!(
                    "Failed to start Filebeat with config: {}\nStdout: {}\nStderr: {}",
                    filebeat_config_path, stdout, stderr
                ).into());
            }
            Ok(())
        }
        Err(e) => {
            // 如果启动命令本身失败，打印出错误信息
            Err(format!("Failed to execute command: {}", e).into())
        }
    }
}



pub(crate) async fn get_and_restart_container(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 执行获取容器 ID 的命令，使用传入的 service_name 来替代固定的 "filebeat"
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("docker ps | grep '{}' | awk '{{print $1}}'", service_name))
        .output()
        .await?;

    if !output.status.success() {
        return Err(format!("Command failed with status: {}", output.status).into());
    }

    // 从命令输出中提取容器 ID
    let container_id = str::from_utf8(&output.stdout)?.trim().to_string();
    info!("Container ID for '{}': {}", service_name, container_id);

    // 执行 docker restart 命令重启容器
    let restart_output = Command::new("docker")
        .arg("restart")
        .arg(&container_id)
        .output()
        .await?;

    if !restart_output.status.success() {
        return Err(format!("Failed to restart container: {}", container_id).into());
    }

    info!("Successfully restarted container: {}", container_id);
    Ok(())
}

/// 执行多层 grep 命令，支持通过管道进行逐层查找
// pub(crate) async fn grep_multiple_layers(path: &str, patterns: Vec<String>) -> Result<String, Box<dyn std::error::Error>> {
//     // 确保 patterns 至少有一个元素
//     if patterns.is_empty() {
//         return Err("No patterns provided".into());
//     }
//
//     // 初始化 shell 命令，首先处理第一个模式
//     let mut cmd = format!("grep -a '{}'", patterns[0]);
//
//     // 为每个模式添加 grep 命令并连接管道
//     for pattern in patterns.iter().skip(1) {
//         cmd = format!("{} | grep -a '{}'", cmd, pattern);
//     }
//
//     // 将路径添加到命令的末尾
//     cmd = format!("{} {}", cmd, path);
//
//     // 使用 sh 执行命令
//     let mut current_command = Command::new("sh");
//     current_command.arg("-c").arg(cmd);
//
//     // 执行命令并获取输出
//     let output = current_command.output().await;
//
//     match output {
//         Ok(output) => {
//             if !output.status.success() {
//                 // 如果命令失败，打印标准输出和标准错误信息
//                 let stdout = String::from_utf8_lossy(&output.stdout);
//                 let stderr = String::from_utf8_lossy(&output.stderr);
//                 return Err(format!(
//                     "grep command failed with status: {}\nStdout: {}\nStderr: {}",
//                     output.status, stdout, stderr
//                 ).into());
//             }
//
//             // 返回命令的标准输出作为结果
//             let result = String::from_utf8_lossy(&output.stdout).to_string();
//             Ok(result)
//         }
//         Err(e) => {
//             // 如果执行命令本身失败，打印错误信息
//             Err(format!("Failed to execute grep command: {}", e).into())
//         }
//     }
// }

pub(crate) async fn grep_multiple_layers(path: &str, patterns: Vec<String>) -> Result<String, Box<dyn std::error::Error>> {
    // 初始化命令
    let mut command = String::new();

    // 拼接第一个 grep 命令
    command.push_str(&format!("grep -a {} {}", patterns[0], path));

    // 拼接后续的 grep 命令
    for pattern in patterns.iter().skip(1) {
        command.push_str(&format!(" | grep -a {}", pattern));
    }

    // 执行拼接后的命令
    let output = Command::new("bash")
        .arg("-c")
        .arg(command) // 使用 bash 执行拼接的命令字符串
        .output()
        .await?;

    // 获取最终的输出并转换为字符串
    let result = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(result)
}
