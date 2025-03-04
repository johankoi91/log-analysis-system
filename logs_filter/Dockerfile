# 使用 Ubuntu 24.04 作为基础镜像
FROM --platform=linux/amd64 rust:latest as builder

# 设置工作目录
WORKDIR /usr/src/logs_filter

# 复制 Rust 项目文件
COPY . .


# 构建 Rust 项目
RUN cargo build --release

# 使用 Ubuntu 24.04 作为运行时基础镜像
FROM --platform=linux/amd64 ubuntu:24.04

# 复制构建好的 Rust 可执行文件到容器
COPY --from=builder /usr/src/logs_filter/target/release/logs_filter /usr/local/bin/

# 设置 CONFIG_FILE_PATH 环境变量
COPY ./config /usr/src/logs_filter/config
ENV CONFIG_FILE_PATH="/usr/src/logs_filter/config/config.yaml"

USER root
# 复制启动脚本并设置可执行权限
COPY start.sh /start.sh
RUN chmod +x /start.sh

# 运行 Rust 应用
CMD ["/start.sh"]
