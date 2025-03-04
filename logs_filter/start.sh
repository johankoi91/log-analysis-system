#!/bin/bash

set -e  # 遇到错误时退出

# 启动 Rust 可执行文件（作为后台进程）
echo "Starting Rust application..."
/usr/local/bin/logs_filter &

# 启动 Logstash
echo "Starting Logstash..."
exec /usr/share/logstash/bin/logstash
