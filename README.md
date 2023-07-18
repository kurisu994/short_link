# 使用AXUM构建短链接服务

使用axum构建高性能的短链服务，实现把长链接转化成长度固定的短链接。

## 短链原理
通过雪花算法给每个链接生成一个唯一id,然后把生成的id转成62进制字符串以达到压缩字符串长度的目的,同时储存这次结果。当请求访问时通过62进制转10进制得到id，通过ID找到原始地址，最后重定向到该地址。

 - 持久化和缓存
 在整个过程中会使用mysql来保存链接数据，使用redis缓存数据以提高性能
 - 防止相同地址多次生成
 相同的地址是不需要重复生成短链，因此会对源地址进行sha256计算得到43位的哈希码，通过对比这个哈希码来判断是否已有重复的数据。选择sha256目的也只是降低hash碰撞的概率，当然可以考虑性能更好的算法。

## 运行项目
### 直接启动
本项目会使用到mysql和redis，所以在启动前请确保mysql和redis已经正确安装。 
1. 目前需要手动创建表，ddl语句可以在 `./sql/ddl.sql`中查看 。 
2. 接着是根据实际情况修改项目配置 `./application.yaml` 。 

到这里就可以直接运行启动命令了。
```shell
cargo run
```
### docker运行
```shell
// 打包镜像
docker build -t short-link:latest -f ./Dockerfile --no-cache .
// 运行镜像
docker run -d  \
  --name short-link \
  --link mysql \
  --link redis \
  -u root \
  -e DATABASE_URL=mysql://root:123456@mysql:3306/short_link \
  -e REDIS_URL=redis://redis:6379 \
  -p 8008:8008 \
  -v "$PWD/application.yaml":/usr/app/application.yaml \
  -v "$PWD/logs":/usr/app/logs \
  short-link:latest
```
这里的`--link mysql`和`--link redis`根据实际情况自行调整。 

docker接收`DATABASE_URL`和`REDIS_URL`这两个环境变量来指定数据库连接地址和redis地址，并且这两个环境变量的优先级会高于配置文件的地址。

### 计划
1. 新增请求拦截器，对请求参数和响应进行日志打印。
2. 新增页面用于配置和统计
3. 使用定时器定期清除过期的链接数据
