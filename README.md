# mirai DB bot

提供DB操作接口的mirai Bot

# INSTALL

## 安装mirai与mcl

[mirai](https://github.com/mamoe/mirai)
[mcl](https://github.com/iTXTech/mirai-console-loader)

安装http插件

[mirai-http](https://github.com/project-mirai/mirai-api-http)

## 配置mirai-http

使用本地8080端口。由于现阶段为加入bot并未加入verifyKey，请拒绝来自外部的8080端口访问

```
# /mirai/config/net.mamoe.mirai-api-http/setting.yml
adapters: 
  - http
  - ws
debug: false
enableVerify: false
verifyKey: KEYNAVI114514
singleMode: true
cacheSize: 4096
adapterSettings: 
    ws:
        host: localhost
        port: 8080
        reservedSyncId: -1
    http:
        host: localhost
        port: 8080
        cors: [*]
```

## 编译Bot

需要安装rust

```bash
cargo build
```

## 配置bot

```
# /bot/config.json
{
    "id": "your_qq",
    "verify_key": "",
    "admin_id": "admin_qq",
    "db_url": "mysql://username:password@host/dbname",
    "worker_amount": 4,
    "instructions": [
        {
            "command": "new",
            "is_public": false,
            "params": [
                "age",
                "sex"
            ],
            "content": "INSERT INTO testtable (id, age, sex) VALUES ({id}, {age}, \"{sex}\")",
            "description": "add a new row"
        },
        {
            "command": "delete",
            "is_public": false,
            "params": [
            ],
            "content": "DELETE FROM testtable WHERE id = {id}",
            "description": "delete a row"
        },
        {
            "command": "update",
            "is_public": false,
            "params": [
                "age",
                "sex"
            ],
            "content": "UPDATE testtable SET sex = \"{sex}\", age = {age} WHERE id = {id}",
            "description": "update a row"
        }
    ]
}
```
