#!/bin/bash


set -e

CONTAINER_NAME="short-link-postgres"

HOST_PORT=15432
CONTAINER_PORT=5432

POSTGRES_USER=postgres
POSTGRES_PASSWORD=Mygvo0oq0818
POSTGRES_DB=short_link

DATA_DIR=$(pwd)/data/postgres

if [ "$(docker ps -q -f name=$CONTAINER_NAME)" ]; then
    echo "✅ PostgreSQL容器 '$CONTAINER_NAME' 已经在运行"
    echo "🔗 连接信息: postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@localhost:$HOST_PORT/$POSTGRES_DB"
    exit 0
fi

if [ "$(docker ps -aq -f name=$CONTAINER_NAME)" ]; then
    echo "🔄 启动已存在的PostgreSQL容器..."
    docker start $CONTAINER_NAME
    echo "✅ PostgreSQL容器已启动"
    echo "🔗 连接信息: postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@localhost:$HOST_PORT/$POSTGRES_DB"
    exit 0
fi

mkdir -p $DATA_DIR

echo "🚀 启动新的PostgreSQL容器..."
docker run -d \
    --name $CONTAINER_NAME \
    -e POSTGRES_USER=$POSTGRES_USER \
    -e POSTGRES_PASSWORD=$POSTGRES_PASSWORD \
    -e POSTGRES_DB=$POSTGRES_DB \
    -e POSTGRES_INITDB_ARGS="--encoding=UTF-8 --lc-collate=C --lc-ctype=C" \
    -p $HOST_PORT:$CONTAINER_PORT \
    -v $DATA_DIR:/var/lib/postgresql/data \
    postgres:16-alpine

echo "⏳ 等待PostgreSQL启动..."
sleep 5

if [ "$(docker ps -q -f name=$CONTAINER_NAME)" ]; then
    echo "✅ PostgreSQL容器启动成功!"
    echo "🔗 连接信息: postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@localhost:$HOST_PORT/$POSTGRES_DB"
    echo "📁 数据目录: $DATA_DIR"
else
    echo "❌ PostgreSQL容器启动失败"
    echo "📋 查看日志: docker logs $CONTAINER_NAME"
    exit 1
fi