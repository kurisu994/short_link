#!/bin/bash

set -e

CONTAINER_NAME="short-link-postgres"

if [ ! "$(docker ps -aq -f name=$CONTAINER_NAME)" ]; then
    echo "ℹ️  PostgreSQL容器 '$CONTAINER_NAME' 不存在"
    exit 0
fi

if [ "$(docker ps -q -f name=$CONTAINER_NAME)" ]; then
    echo "🛑 停止PostgreSQL容器..."
    docker stop $CONTAINER_NAME
    echo "✅ PostgreSQL容器已停止"
else
    echo "ℹ️  PostgreSQL容器已经停止"
fi

read -p "🗑️  是否要删除PostgreSQL容器? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🗑️  删除PostgreSQL容器..."
    docker rm $CONTAINER_NAME
    echo "✅ PostgreSQL容器已删除"

    DATA_DIR=$(pwd)/data/postgres
    if [ -d "$DATA_DIR" ]; then
        read -p "🗑️  是否要删除数据库数据目录 $DATA_DIR? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo "🗑️  删除数据库数据..."
            rm -rf $DATA_DIR
            echo "✅ 数据库数据已删除"
        fi
    fi
fi

echo "🏁 操作完成"