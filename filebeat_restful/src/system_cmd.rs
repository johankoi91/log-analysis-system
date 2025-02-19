use tokio::process::Command;
use std::str;


pub(crate) async fn start_filebeat(filebeat_config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let filebeat_output = Command::new("/usr/local/bin/filebeat")
        .arg("-e")
        .arg("-c")
        .arg(filebeat_config_path)
        .output()
        .await?;

    if !filebeat_output.status.success() {
        return Err(format!("Failed to start Filebeat with config: {}", filebeat_config_path).into());
    }
    println!("Successfully started Filebeat with config: {}", filebeat_config_path);
    Ok(())
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
    println!("Container ID for '{}': {}", service_name, container_id);

    // 执行 docker restart 命令重启容器
    let restart_output = Command::new("docker")
        .arg("restart")
        .arg(&container_id)
        .output()
        .await?;

    if !restart_output.status.success() {
        return Err(format!("Failed to restart container: {}", container_id).into());
    }

    println!("Successfully restarted container: {}", container_id);
    Ok(())
}