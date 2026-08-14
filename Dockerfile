# Tailscale 设备状态面板 - Rust 版本
# 构建: 传入 BINARY 指定架构二进制 (tailscale-amd64 / tailscale-arm64)
# 例 (arm64 在 Pi 上): docker build --build-arg BINARY=tailscale-arm64 -t tailscale-api .
# 例 (amd64 在 x86 机器): docker build --build-arg BINARY=tailscale-amd64 -t tailscale-api .

FROM alpine:3.20
ARG BINARY
# 版本标签 (强制新 digest, 避免 ghcr 平台缓存 bug)
LABEL org.opencontainers.image.version="1.1.0"
# iputils: 提供 ping (连通性测试)
RUN apk add --no-cache iputils
# 对应架构的静态二进制
COPY bin/${BINARY} /tailscale-api
# 前端文件
COPY templates /templates
COPY static /static

EXPOSE 8091
ENTRYPOINT ["/tailscale-api"]
