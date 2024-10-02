# Crane - 快速且安全的 .tar.gz 部署工具

![Crane](logo.jpg)

[English](README.md) | 简体中文

Crane 是一个使用 Rust 编写的简单、快速且安全的工具，用于下载和部署您的 `.tar.gz` 归档文件，无需服务器密码或密钥。它旨在提供一种快速方便的方式，将您的软件或文件部署到服务器上，同时避免通常的身份验证和授权复杂性。

## 功能特点

1. **快速**: 使用 Rust 编写，**Crane** 提供仅 3MB 的轻量级二进制大小和仅 88MB 的 Docker 镜像，确保快速高效的部署。


2. **简单**: 只需一个 GET 请求，Crane 即可下载并解压您的 `.tar.gz` 文件，使部署过程变得轻松。


3. **安全**: Crane 无需服务器凭据或密钥即可运行，提供了额外的安全层。其 API 支持 API 密钥保护，并确保文件仅写入指定路径，防止未经授权的访问。

## 安装说明


1. 使用 Docker

```bash
docker run -e API_KEY={YOUR_SUPER_SECURE_API_KEY} -p 8594:8594 -v /dist_path_on_host/:/dist/ wangyucode/crane:0.1.0
```

2. 使用 docker-compose

```yaml
services:
  crane:
    image: wangyucode/crane:0.1.0
    environment:
      - API_KEY={YOUR_SUPER_SECURE_API_KEY}
    ports:
      - 8594:8594
    volumes:
      - /dist_path_on_host/:/dist/
```

3. 使用二进制文件
```
git clone https://github.com/wangyucode/crane
cargo build --release
./target/release/crane
```


## 使用方法

```bash
curl -H "X-Api-Key: {YOUR_SUPER_SECURE_API_KEY}" http://{your_server_address}:8594/?url=https://example.com/file.tar.gz
```

因此，您可以在 CI/CD 管道中使用它。

### Github actions example:
```yaml
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      ...
      - name: use Crane to deploy
        run: |
          curl -H "X-Api-Key: ${{secrets.CRANE_API_KEY}}" http://${secrets.SERVER_ADDRESS}:8594/?url=https://github.com/your-repo/your-repo/releases/download/v1.0.0/dist.tar.gz
      ...
```

> **Waring**: 警告: API_KEY 是必需的，如果未设置，Crane 将无法启动。请将使用强密码。

## 路线图

- [ ] 当新的部署被触发时，取消上一个部署
- [ ] 支持选项，覆盖标志