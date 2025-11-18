# CLAUDE.md

此文件为 Claude Code (claude.ai/code) 在此代码库中工作时提供指导。

## 项目概述

这是一个使用 Rust 和 AXUM Web 框架构建的高性能短链接服务。它使用雪花 ID 算法和 base62 编码将长链接转换为短链接，使用 PostgreSQL 进行持久化存储，使用 Redis 进行缓存。

## Rust 版本和框架版本

- **Rust**: 1.91.0 (2025-10-28)
- **AXUM**: 0.7.x (主要 Web 框架)
- **SQLx**: 0.8.x (数据库 ORM)
- **Redis**: 0.26.x (缓存客户端)
- **Tower-HTTP**: 0.6.x (HTTP 中间件)
- **Validator**: 0.18.x (数据验证)
- **Base64**: 0.22.x (编码工具)
- **bb8**: 0.8.x (连接池)
- **bb8-redis**: 0.15.x (Redis 连接池)
- **http-body-util**: 0.1.x (HTTP body 工具)
- **Tokio**: 1.0.x (异步运行时)
- **Serde**: 1.0.x (序列化/反序列化)
- **Serde YAML**: 0.9.x (YAML 支持)
- **Chrono**: 0.4.x (日期时间处理)
- **SHA2**: 0.10.x (哈希算法)
- **Tracing**: 0.1.x (结构化日志)
- **Tracing Appender**: 0.2.x (日志文件追加)
- **Tracing Subscriber**: 0.3.x (日志订阅器)
- **UUID**: 1.0.x (唯一标识符)
- **Anyhow**: 1.0.x (错误处理)

### 升级注意事项
- **Rust 2024 Edition**: 升级到 2024 edition，享受最新的语言特性和改进
- **AXUM 0.7**: 移除了 `headers` 特性，改用 `http-body-util` 处理 HTTP body
- **服务器启动**: 从 `axum::Server::bind` 改为 `axum::serve` + `tokio::net::TcpListener`
- **中间件**: `Next` 不再需要泛型参数
- **Redis**: 查询操作需要明确的类型注解以避免 never type fallback 警告
- **bb8 连接池**: 使用 bb8 作为 Redis 连接池管理器，提供更好的并发性能
- **Tracing**: 使用 tracing 系列crate进行结构化日志记录，支持文件滚动和多输出
- **UUID**: 启用 fast-rng 和 macro-diagnostics 特性以提升性能和开发体验
- **Serde YAML**: 用于配置文件的解析，支持复杂的嵌套结构
- **静态变量管理**: Rust 2024 对可变静态变量引用有更严格限制，使用 `std::sync::OnceLock` 替代手动管理

# 语言设置
- 所有回答必须使用中文
- 代码注释使用中文
- 文档说明使用中文

## 架构

### 核心组件

- **主应用程序** (`src/main.rs`): 程序入口点，包含服务器设置、中间件、日志记录和优雅关闭
- **配置** (`src/config.rs`): 基于 YAML 的数据库和 Redis 连接配置
- **状态管理** (`src/prepare.rs`): 数据库连接池和应用程序状态初始化
- **ID 生成** (`src/idgen/`): 雪花算法实现，用于生成唯一 ID
- **服务层** (`src/service/`): 链接创建和 URL 解析的业务逻辑
- **处理器** (`src/handle/`): API 和管理端点的 HTTP 路由处理器
- **模型** (`src/pojo/`): 数据结构和数据库实体
- **工具类** (`src/utils/`): 哈希、base62 编码/解码等辅助函数

### 关键设计模式

- **雪花 ID**: 每个链接使用雪花算法获得唯一的 64 位 ID
- **Base62 编码**: ID 转换为 base62 字符串以生成紧凑的短链接
- **SHA256 哈希**: 原始链接进行哈希处理以防止重复生成短链接
- **双层缓存**: Redis 同时缓存 hash->ID 和 ID->URL 映射
- **连接池管理**: 使用 bb8 管理 Redis 连接，sqlx 管理 PostgreSQL 连接

## 常用开发命令

### 构建和运行
```bash
# 标准开发构建和运行
cargo run

# 生产构建
cargo build --release

# 运行测试
cargo test

# 格式化代码（使用 rustfmt.toml 配置）
cargo fmt

# 检查代码而不构建
cargo check
```

### Docker 开发
```bash
# 构建 Docker 镜像
docker build -t short-link:latest -f ./Dockerfile --no-cache .

# 使用 Docker 运行
docker run -d \
  --name short-link \
  --link postgres \
  --link redis \
  -u root \
  -e DATABASE_URL=postgresql://postgres:123456@postgres:5432/short_link \
  -e REDIS_URL=redis://redis:6379 \
  -p 8008:8008 \
  -v "$PWD/application.yaml":/usr/app/application.yaml \
  -v "$PWD/logs":/usr/app/logs \
  short-link:latest
```

## 数据库设置

### PostgreSQL 模式
应用程序需要手动设置数据库。运行 `./sql/ddl.sql` 中的 DDL：
```sql
create table if not exists link_history
(
    id          bigint                             not null primary key,
    origin_url  varchar(4000)                      not null comment '原始的地址',
    link_type   int(3)                                null comment '链接类型 1:短期 2:长期',
    expire_date datetime                           null,
    active      tinyint(1)                         not null comment '是否有效的',
    link_hash   varchar(48)                        not null comment '链接的hash值',
    create_time datetime default CURRENT_TIMESTAMP null,
    update_time datetime default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP,
    constraint link_history_link_hash_uindex unique (link_hash)
);
```

## 配置

### 环境变量（优先级高于配置文件）
- `DATABASE_URL`: PostgreSQL 连接字符串（格式：postgresql://user:password@host:port/database）
- `REDIS_URL`: Redis 连接字符串（格式：redis://[user:password@]host:port）

### 配置文件
应用程序按以下顺序加载配置：
1. `application.local.yaml`（本地覆盖）
2. `application.yaml`（默认配置）

查看 `application.yaml` 了解配置结构，包括数据库连接池设置、Redis 配置和超时设置。

## API 端点

### 公共 API
- `GET /s/:hash` - 重定向到原始 URL（公共短链接访问）

### 管理 API
- `POST /link/create` - 创建新的短链接
  ```json
  {
    "url": "https://example.com",
    "duration": 3600  // 可选的过期时间（秒）
  }
  ```
- `GET /link/list` - 列出现有链接（分页）

## 开发注意事项

### 代码风格
- 使用 `rustfmt.toml` 进行一致的格式化
- 使用 Rust 2024 版本特性
- 导入按 crate 粒度组织

### 错误处理
- 使用 `anyhow` 进行错误传播
- 自定义 `AppError` 类型用于 API 响应
- 使用 tracing 进行全面的日志记录

### 测试
- 本项目不需要创建测试脚本
- 只做基础类型检查和编译检查
- 可使用 `cargo check` 进行类型检查，`cargo test` 运行测试

### 日志记录
- 使用 tracing 进行结构化日志记录
- 日志同时写入 stdout（控制台）和 `./logs/` 中的滚动文件
- 文件日志按天自动滚动，格式为 `short-link.log.YYYY-MM-DD`
- 控制台日志使用彩色美化格式，文件日志使用紧凑格式
- 请求/响应中间件使用唯一 ID 记录所有 HTTP 流量
- 默认日志级别为 INFO，可通过环境变量 `RUST_LOG` 调整

## 重要实现细节

### ID 生成算法
系统使用修改的雪花算法（YitIdHelper）生成唯一的 64 位 ID，然后进行 base62 编码以创建紧凑的短链接。

### 缓存策略
- **哈希缓存**: `link:hash:{sha256_hash}` -> `{id}`（防止重复 URL 创建）
- **URL 缓存**: `link:origin:uri:{id}` -> `{original_url}`（快速 URL 查找）
- 缓存是写穿透的：数据库写入会触发缓存更新

### 安全考虑
- 使用 validator crate 进行 URL 验证
- SHA256 哈希最小化冲突风险
- 管理端点无身份验证（生产环境建议添加）
- 当前配置对所有来源启用 CORS

