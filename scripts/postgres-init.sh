#!/bin/bash

set -e

# 数据库配置
POSTGRES_USER=postgres
POSTGRES_PASSWORD=Mygvo0oq0818
POSTGRES_DB=short_link
HOST=localhost
PORT=15432

DDL_PATH=$(dirname "$0")/../sql/postgres-ddl.sql

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}🚀 开始初始化PostgreSQL数据库...${NC}"

if [ ! -f "$DDL_PATH" ]; then
    echo -e "${RED}❌ DDL文件不存在: $DDL_PATH${NC}"
    exit 1
fi

echo -e "${YELLOW}🔍 检查PostgreSQL连接...${NC}"
if ! PGPASSWORD=$POSTGRES_PASSWORD psql -h $HOST -p $PORT -U $POSTGRES_USER -d postgres -c "SELECT 1;" > /dev/null 2>&1; then
    echo -e "${RED}❌ 无法连接到PostgreSQL服务器${NC}"
    echo -e "${YELLOW}💡 请确保PostgreSQL容器正在运行: ./scripts/postgres-start.sh${NC}"
    exit 1
fi

echo -e "${YELLOW}📋 检查数据库 '$POSTGRES_DB'...${NC}"
DB_EXISTS=$(PGPASSWORD=$POSTGRES_PASSWORD psql -h $HOST -p $PORT -U $POSTGRES_USER -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname='$POSTGRES_DB'")

if [ "$DB_EXISTS" != "1" ]; then
    echo -e "${YELLOW}📝 创建数据库 '$POSTGRES_DB'...${NC}"
    PGPASSWORD=$POSTGRES_PASSWORD createdb -h $HOST -p $PORT -U $POSTGRES_USER $POSTGRES_DB
    echo -e "${GREEN}✅ 数据库创建成功${NC}"
else
    echo -e "${GREEN}✅ 数据库已存在${NC}"
fi

echo -e "${YELLOW}📝 执行DDL脚本...${NC}"
if PGPASSWORD=$POSTGRES_PASSWORD psql -h $HOST -p $PORT -U $POSTGRES_USER -d $POSTGRES_DB -f "$DDL_PATH"; then
    echo -e "${GREEN}✅ DDL脚本执行成功${NC}"
else
    echo -e "${RED}❌ DDL脚本执行失败${NC}"
    exit 1
fi

echo -e "${YELLOW}🔍 验证表结构...${NC}"
TABLE_EXISTS=$(PGPASSWORD=$POSTGRES_PASSWORD psql -h $HOST -p $PORT -U $POSTGRES_USER -d $POSTGRES_DB -tAc "SELECT 1 FROM information_schema.tables WHERE table_name='link_history'")

if [ "$TABLE_EXISTS" = "1" ]; then
    echo -e "${GREEN}✅ 表 'link_history' 创建成功${NC}"
else
    echo -e "${RED}❌ 表 'link_history' 创建失败${NC}"
    exit 1
fi

# 显示表结构
echo -e "${YELLOW}📋 表结构信息:${NC}"
PGPASSWORD=$POSTGRES_PASSWORD psql -h $HOST -p $PORT -U $POSTGRES_USER -d $POSTGRES_DB -c "\d link_history"

echo -e "${GREEN}🎉 PostgreSQL数据库初始化完成!${NC}"
echo -e "${YELLOW}🔗 连接信息: postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@$HOST:$PORT/$POSTGRES_DB${NC}"