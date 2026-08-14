# Tailscale 面板

[English](README.md) | [简体中文](README.zh-CN.md) | [Русский](README.ru-RU.md)

一个轻量级的 Tailscale 网络节点状态监控面板，局域网内任意设备（手机、平板、电脑）均可访问。

## 功能

- 📊 **设备列表**：在线/离线状态、系统类型、IPv4/IPv6、MagicDNS 域名、所属用户
- 🔍 **搜索与复制**：搜索设备，点击 IP 一键复制
- 📡 **连通性测试**：一键 ping 任意设备，显示延迟
- 🌓 **主题模式**：自动（按时间段）/ 跟随系统 / 白天 / 夜间
- 🔄 **自动刷新**：每 30 秒
- 🖼️ **自定义背景**：上传图片、可调透明度、卡片半透明

## 界面截图

![Tailscale 设备面板](docs/screenshot.png)

## 技术特点

- **Rust 后端**（Axum 框架）——编译为单个静态二进制，无运行时依赖
- **多架构支持**：预构建 `linux/amd64` 和 `linux/arm64`
- **镜像极小**：仅约 19MB 单个静态二进制
- 通过本地 socket API 与 `tailscaled` 通信（无需 tailscale 二进制）

## 快速开始

### 方式一：拉取预构建镜像（推荐）

无需本地编译（多架构：amd64 + arm64）：

**第 1 步：拉取镜像**

```bash
docker pull ghcr.io/saves24/tailscale-api:latest
```

**第 2 步：运行容器**

```bash
docker run -d --name tailscale-api \
  --network host \
  -v /var/run/tailscale:/var/run/tailscale \
  --restart unless-stopped \
  ghcr.io/saves24/tailscale-api:latest
```

> 使用 `host` 网络模式（读取宿主机网络统计）。访问：`http://<宿主机>:8091/panel`

### 方式二：源码构建

```bash
# arm64（在 Pi 上）：
docker build --build-arg BINARY=tailscale-arm64 -t tailscale-api .
# amd64（在 x86 机器）：
docker build --build-arg BINARY=tailscale-amd64 -t tailscale-api .

# 或使用 compose：
docker compose up -d
```

然后访问：`http://<主机>:8091/panel`

## 依赖

- Docker + Docker Compose
- 宿主机已运行 Tailscale（tailscaled）
- 通过挂载 `/var/run/tailscale` socket 与 tailscaled 通信

## 配置

| 环境变量 | 默认值 | 说明 |
|---|---|---|
| `CACHE_TTL` | `5` | `tailscale status` 缓存秒数 |

## API

| 接口 | 说明 |
|---|---|
| `GET /` | 服务状态（JSON） |
| `GET /devices` | 设备列表（JSON） |
| `GET /network` | 网络统计（JSON） |
| `GET /ping/<ip>` | 连通性测试（JSON） |
| `GET /panel` | 网页面板 |

## 项目结构

```
├── src/main.rs          # Rust 应用 (Axum)
├── Cargo.toml           # Rust 依赖
├── templates/panel.html # HTML 模板 (编译时嵌入)
├── static/              # CSS / JS (运行时读取)
├── Dockerfile           # 容器构建 (ARG BINARY 指定架构)
├── docker-compose.yml   # Compose 配置
└── .gitignore
```

## License

MIT
