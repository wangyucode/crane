<div align="center">
  <img src="logo.jpg" width=256></img>
  <p><strong>Crane - Fast and Secure .tar.gz Deployment</strong></p>

  English | [简体中文](README_Zh-CN.md)

</div>

Crane is a simple, fast, and secure tool write in Rust for downloading and deploying your `.tar.gz` archive files without the need for server passwords or keys. It was designed to provide a quick and convenient way to deploy your software or files to a server without the usual complexities of authentication and authorization.

## Features
1. **Fast:** Written in Rust, **Crane** offers a lightweight binary size of just 3MB and a Docker image of only 88MB, ensuring quick and efficient deployment.

2. **Simple:** With a single GET request, Crane can download and unzip your `.tar.gz` file, making the deployment process effortless.

3. **Secure:** Crane operates without the need for server credentials or keys, providing an additional layer of security. Its API supports API keys for protection, and it ensures that files are only written to the specified path, preventing unauthorized access.

## Installation

Crane will decompress the `.tar.gz` file to the `/dist/` directory. So when using Docker, just mount the host directory you want to deploy to the `/dist/` directory of the container.

1. use Docker

```bash
docker run -e API_KEY={YOUR_SUPER_SECURE_API_KEY} -p 8594:8594 -v /dist_path_on_host/:/dist/ wangyucode/crane:0.1.2
```

2. use docker-compose

```yaml
services:
  crane:
    image: wangyucode/crane:0.1.2
    environment:
      - API_KEY={YOUR_SUPER_SECURE_API_KEY}
    ports:
      - 8594:8594
    volumes:
      - /dist_path_on_host/:/dist/
```

3. use binary
```
git clone https://github.com/wangyucode/crane
cargo build --release
./target/release/crane
```


## Usage

```bash
curl -H "X-Api-Key: {YOUR_SUPER_SECURE_API_KEY}" http://{your_server_address}:8594/deploy?url=https://example.com/file.tar.gz
```

so you can use it in your CI/CD pipeline. 

### Github actions example:
```yaml
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      ...
      - name: use Crane to deploy
        run: |
          curl -H "X-Api-Key: ${{secrets.CRANE_API_KEY}}" http://${secrets.SERVER_ADDRESS}:8594/deploy?url=https://github.com/your-repo/your-repo/releases/download/v1.0.0/dist.tar.gz
      ...
```

> **Waring**: API_KEY is required, if not set, Crane will not start. and please set it as a strong secret.

### TODO list

- [ ] Cancel last deployment when new deployment is triggered.
- [ ] Support options, override flag.

## Other Options

If you're looking for a GitHub action to deploy files via `sftp` using server keys, you can try [wangyucode/sftp-upload-action](https://github.com/wangyucode/sftp-upload-action)
