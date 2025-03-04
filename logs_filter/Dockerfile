# 使用 Rust 的官方镜像来构建 Rust 应用
FROM --platform=linux/amd64 rust:latest as builder

# 设置工作目录
WORKDIR /usr/src/logs_filter

# 复制 Rust 项目文件
COPY . .

# 构建 Rust 项目
RUN cargo build --release

# 使用 Logstash 官方镜像作为基础镜像
FROM --platform=linux/amd64 docker.elastic.co/logstash/logstash:8.17.0

# 设置工作目录
WORKDIR /usr/src/logs_filter

# 复制构建好的 Rust 可执行文件到容器
COPY --from=builder /usr/src/logs_filter/target/release/logs_filter /usr/local/bin/

# 设置 LOG_FILE_PATH 环境变量
COPY ./config /usr/src/logs_filter/config
ENV LOG_FILE_PATH="/usr/src/logs_filter/config/config.yaml"

# 复制 Logstash 配置文件到容器
COPY ./logstash-docker/logstash.conf /usr/share/logstash/pipeline/logstash.conf
COPY ./logstash-docker/logstash.yml /usr/share/logstash/config/logstash.yml
COPY ./logstash-docker/jvm.options /usr/share/logstash/config/jvm.options

# 暴露 Logstash 的端口
EXPOSE 9600
EXPOSE 5044

# 设置适当的权限
USER root
RUN ls -l /usr/share/logstash && \
    chown -R root:root /usr/share/logstash

# 复制启动脚本并设置可执行权限
COPY start.sh /start.sh
RUN chmod +x /start.sh

# 启动时运行 start.sh 脚本
CMD ["/start.sh"]
