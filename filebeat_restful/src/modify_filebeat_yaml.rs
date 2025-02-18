use serde::{Deserialize, Serialize};
use serde_yaml::{Value, from_reader, to_writer};
use std::fs::File;

// 定义结构体来对应 YAML 文件中的数据结构
#[derive(Debug, Serialize, Deserialize)]
struct LogConfig {
    r#type: String,
    enabled: bool,
    paths: Vec<String>,
    fields: Fields,
    scan_frequency: String,
    close_inactive: String,
    fields_under_root: bool,
}

// 结构体用于嵌套字段 "fields"
#[derive(Debug, Serialize, Deserialize)]
struct Fields {
    service: String,
    hostname: String,
}

// 修改 YAML 文件中的 paths, service 和 hostname 字段
pub fn modify_yaml(file_path: &str, new_paths: Vec<String>, new_service: String, new_hostname: String) -> Result<(), Box<dyn std::error::Error>> {
    // 打开文件并解析 YAML 内容
    let file = File::open(file_path)?;
    let mut data: Vec<LogConfig> = serde_yaml::from_reader(file)?;

    // 假设我们修改第一个配置项的 `paths`, `service` 和 `hostname`
    if let Some(config) = data.get_mut(0) {
        // 修改 `paths` 字段
        config.paths = new_paths;
        // 修改 `service` 字段
        config.fields.service = new_service;
        // 修改 `hostname` 字段
        config.fields.hostname = new_hostname;
    }

    // 打开文件进行写回修改后的内容
    let file = File::create(file_path)?;
    serde_yaml::to_writer(file, &data)?;

    Ok(())
}

// 使用 `serde_yaml::Value` 进行修改（动态结构）
pub fn modify_yaml_dynamic(file_path: &str, new_paths: Vec<String>, new_service: String, new_hostname: String) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let mut data: Value = from_reader(file)?;

    // 修改第一个配置项的 paths, service 和 hostname
    if let Some(config) = data.get_mut(0) {
        if let Some(paths) = config.get_mut("paths") {
            *paths = Value::Sequence(new_paths.into_iter().map(Value::String).collect());
        }
        if let Some(fields) = config.get_mut("fields") {
            if let Some(service) = fields.get_mut("service") {
                *service = Value::String(new_service);
            }
            if let Some(hostname) = fields.get_mut("hostname") {
                *hostname = Value::String(new_hostname);
            }
        }
    }

    // 写回修改后的数据
    let file = File::create(file_path)?;
    to_writer(file, &data)?;

    Ok(())
}
//
// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let file_path = "config.yml"; // 指定 YAML 文件路径
//
//     // 修改的字段值
//     let new_paths = vec![
//         "/new/path/to/log1.log".to_string(),
//         "/new/path/to/log2.log".to_string(),
//     ];
//     let new_service = "NewRTC".to_string();
//     let new_hostname = "NewMachine".to_string();
//
//     // 使用结构体实现修改
//     modify_yaml(file_path, new_paths, new_service, new_hostname)?;
//
//     println!("YAML 文件已更新。");
//
//     // 如果想使用动态方式修改，替换上述行：
//     // modify_yaml_dynamic(file_path, new_paths, new_service, new_hostname)?;
//
//     Ok(())
// }
