#!/bin/bash

# 启动 Rust 应用程序
/usr/local/bin/filebeat_restful &

# 启动 Filebeat 容器
filebeat -e -c /usr/share/filebeat/filebeat.yml
