# 使用AXUM构建短链接服务

使用[axum](https://github.com/tokio-rs/axum)构建高性能的短链服务，实现把长链接转化成长度固定的短链接。

## 短链原理
通过[雪花算法](https://gitee.com/yitter/idgenerator)给每个链接生成一个唯一id,然后把生成的id转成62进制字符串以达到压缩字符串长度的目的,同时储存这次结果。当请求访问时通过62进制转10进制得到id，通过ID找到原始地址，最后重定向到该地址。

 - 持久化和缓存
 在整个过程中会使用PostgreSQL来保存链接数据，使用redis缓存数据以提高性能
 - 防止相同地址多次生成
 相同的地址是不需要重复生成短链，因此会对源地址进行sha256计算得到43位的哈希码，通过对比这个哈希码来判断是否已有重复的数据。选择sha256目的也只是降低hash碰撞的概率，当然可以考虑性能更好的算法。

## 技术栈
- **Rust** 1.91.0 (2024 Edition)
- **AXUM** 0.7.x - 高性能异步Web框架
- **PostgreSQL** - 主数据库
- **Redis** - 缓存系统
- **SQLx** 0.8.x - 异步数据库ORM
- **bb8** - 连接池管理
- **Serde** - 序列化/反序列化
- **Tracing** - 结构化日志记录
- **UUID** - 唯一标识符生成

## 运行项目

### 直接启动
本项目会使用到PostgreSQL和Redis，所以在启动前请确保这两个服务已经正确安装。

1. **数据库初始化**：目前需要手动创建表，DDL语句可以在 `./sql/ddl.sql`中查看
2. **配置修改**：根据实际情况修改项目配置 `./application.yaml`
3. **启动应用**：
```shell
cargo run
```

### Docker运行
```shell
# 构建镜像
docker build -t short-link:latest -f ./Dockerfile --no-cache .

# 运行镜像
docker run -d  \
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

**注意事项**：
- `--link postgres` 和 `--link redis` 根据实际情况调整
- Docker 接收 `DATABASE_URL` 和 `REDIS_URL` 环境变量来指定连接地址
- 环境变量的优先级高于配置文件

## API文档

### 公共API
- `GET /s/{hash}` - 重定向到原始URL

### 管理API
- `POST /link/create` - 创建短链接
  ```json
  {
    "url": "https://example.com",
    "duration": 3600
  }
  ```
- `GET /link/list` - 获取链接列表（支持分页）
  ```bash
  GET /link/list?page=1&page_size=10
  ```

## 主要特性

✅ **高性能**：基于Rust和AXUM异步框架，支持高并发
✅ **智能缓存**：Redis双层缓存策略（hash->ID, ID->URL）
✅ **防重复生成**：SHA256哈希检测相同URL
✅ **结构化日志**：详细的请求/响应日志，支持JSON格式化输出
✅ **优雅关闭**：支持信号处理和资源清理
✅ **CORS支持**：跨域请求支持
✅ **分页查询**：链接列表支持高效分页

## 开发工具

### 常用命令
```bash
# 运行项目
cargo run

# 构建发布版本
cargo build --release

# 类型检查
cargo check

# 代码格式化
cargo fmt

# 运行测试
cargo test
```

### 日志记录
- **控制台日志**：彩色美化格式，便于开发调试
- **文件日志**：按天滚动，存储在 `./logs/` 目录
- **请求跟踪**：每个请求都有唯一ID，支持完整的请求-响应链路跟踪
- **JSON格式化**：自动识别并格式化JSON响应，便于复制使用

## 项目结构
```
src/
├── main.rs              # 程序入口，中间件配置
├── config.rs            # 配置管理
├── prepare.rs           # 状态初始化，数据库连接
├── handle/              # HTTP处理器
│   ├── admin.rs         # 管理API
│   └── api.rs           # 公共API
├── service/             # 业务逻辑层
│   ├── link_service.rs  # 链接服务
│   └── link_base_service.rs # 数据访问层
├── pojo/                # 数据模型
├── idgen/               # ID生成器
├── utils/               # 工具函数
└── types.rs             # 类型定义
```

## 未来计划
- [x] ✅ 请求拦截器，对请求参数和响应进行日志打印
- [ ] 📋 管理界面，用于配置和统计
- [ ] 📋 定时任务，定期清除过期的链接数据
- [ ] 📋 链接访问统计和分析
- [ ] 📋 批量生成短链接功能
