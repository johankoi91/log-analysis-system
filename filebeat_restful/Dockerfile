# 使用 Rust 的官方镜像来构建 Rust 应用
#FROM rust:latest as builder
FROM --platform=linux/amd64 rust:latest as builder


# 设置工作目录
WORKDIR /usr/src/filebeat_restful

# 复制 Rust 项目文件
COPY . .

# 构建 Rust 项目
RUN cargo build --release

# 使用 Filebeat 官方镜像作为基础镜像
FROM --platform=linux/amd64 docker.elastic.co/beats/filebeat:8.17.0

# 复制构建好的 Rust 可执行文件到容器
COPY --from=builder /usr/src/filebeat_restful/target/release/filebeat_restful /usr/local/bin/

# 设置 LOG_FILE_PATH 环境变量
COPY ./config/log.yaml /usr/src/filebeat_restful/log.yaml
ENV LOG_FILE_PATH="/usr/src/filebeat_restful/log.yaml"

# 复制 Filebeat 配置文件和日志目录
COPY ./config/filebeat/filebeat.yml /usr/share/filebeat/filebeat.yml
ENV FILEBEAT_CONFIG_MAIN_PATH="/usr/share/filebeat/filebeat.yml"
COPY ./config/filebeat/inputs.d /usr/share/filebeat/inputs.d
ENV FILEBEAT_CONFIG_LOG_PATH="/usr/share/filebeat/inputs.d/log.yml"


# 确保以 root 用户执行这些操作
USER root

# 设置适当的权限
RUN ls -l /usr/share/filebeat && \
    chown -R root:root /usr/share/filebeat

# 创建启动脚本来启动 Rust 项目和 Filebeat 容器
COPY start.sh /start.sh
RUN chmod +x /start.sh

# 启动时运行 start.sh 脚本
CMD ["/start.sh"]