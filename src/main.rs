// Tailscale 设备状态面板 - Rust 版本
// 通过 tailscaled.sock 本地 API 通信, 无外部依赖, 静态链接 musl
// 架构: amd64 + arm64 双架构 (编译时指定 target)
use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    sync::Mutex,
    time::{Duration, Instant},
};

const SOCKET_PATH: &str = "/var/run/tailscale/tailscaled.sock";
const CACHE_TTL: Duration = Duration::from_secs(5);
const PORT: u16 = 8091;

// 环境变量读取 (带默认值)
fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

// 简单状态缓存
static CACHE: Mutex<Option<(Instant, Value)>> = Mutex::new(None);

// ---------- tailscaled socket API ----------

fn sock_request(endpoint: &str) -> Result<Value, String> {
    let socket_path = env_or("SOCKET_PATH", SOCKET_PATH);
    let mut stream = UnixStream::connect(&socket_path)
        .map_err(|e| format!("连接 socket 失败: {e}"))?;

    let req = format!(
        "GET {} HTTP/1.1\r\nHost: local-tailscaled.sock\r\nConnection: close\r\n\r\n",
        endpoint
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("写入请求失败: {e}"))?;

    let mut data = Vec::new();
    stream
        .read_to_end(&mut data)
        .map_err(|e| format!("读取响应失败: {e}"))?;

    let resp = String::from_utf8_lossy(&data).to_string();
    let (header, body) = resp
        .split_once("\r\n\r\n")
        .ok_or_else(|| "无效响应格式".to_string())?;

    // chunked 解码
    let body = if header.to_lowercase().contains("transfer-encoding: chunked") {
        decode_chunked(body)
    } else {
        body.to_string()
    };

    serde_json::from_str(&body).map_err(|e| format!("JSON 解析失败: {e}"))
}

fn decode_chunked(body: &str) -> String {
    let mut result = String::new();
    let mut pos = 0;
    while pos < body.len() {
        let line_end = body[pos..].find("\r\n").map(|i| pos + i);
        let Some(line_end) = line_end else { break };
        let size_str = &body[pos..line_end];
        let Ok(size) = usize::from_str_radix(size_str.trim(), 16) else {
            break;
        };
        if size == 0 {
            break;
        }
        let start = line_end + 2;
        if start + size > body.len() {
            break;
        }
        result.push_str(&body[start..start + size]);
        pos = start + size + 2;
    }
    result
}

fn get_status() -> Result<Value, String> {
    let mut cache = CACHE.lock().unwrap();
    if let Some((time, data)) = &*cache {
        if time.elapsed() < CACHE_TTL {
            return Ok(data.clone());
        }
    }
    let data = sock_request("/localapi/v0/status")?;
    *cache = Some((Instant::now(), data.clone()));
    Ok(data)
}

// ---------- 数据处理 ----------

fn get_os_icon(os: &str) -> String {
    let os = os.to_lowercase();
    let svg = if os.contains("linux") {
        r#"<path d="M12 2C9.2 2 7 4.2 7 7v5H5c-1.1 0-2 .9-2 2v4c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2v-4c0-1.1-.9-2-2-2h-2V7c0-2.8-2.2-5-5-5zm-2 5.5c0-.6.4-1 1-1s1 .4 1 1-.4 1-1 1-1-.4-1-1zM14 16h-4v-1.5h4V16zm2-3.5H8V11h8v1.5z"/>"#
    } else if os.contains("windows") {
        r#"<path d="M3 5.5L10.5 4.5V11.5H3V5.5zM11.5 4.3L21 3V11.5H11.5V4.3zM3 12.5H10.5V19.5L3 18.5V12.5zM11.5 12.5H21V21L11.5 19.7V12.5z"/>"#
    } else if os.contains("android") {
        r#"<path d="M6 18c0 .6.4 1 1 1h1v3c0 .6.4 1 1 1s1-.4 1-1v-3h4v3c0 .6.4 1 1 1s1-.4 1-1v-3h1c.6 0 1-.4 1-1v-8H6v8zM15.5 6.5c.3-.2.3-.6.1-.9-.2-.3-.6-.3-.9-.1-.3.2-.3.6-.1.9.2.3.6.3.9.1zm-7 0c.3.2.7.2.9-.1.2-.3.2-.6-.1-.9-.3-.2-.7-.2-.9.1-.2.3-.2.6.1.9zM7 5.5c-1.8 1-3 2.9-3 5h16c0-2.1-1.2-4-3-5l2-3.4c.1-.2 0-.4-.2-.5-.2-.1-.4 0-.5.2L16 5.4c-.7-.3-1.5-.4-2.4-.4h-3.2c-.9 0-1.7.1-2.4.4L5.7 1.8c-.1-.2-.3-.3-.5-.2-.2.1-.3.3-.2.5L7 5.5z"/>"#
    } else if os.contains("ios") {
        r#"<path d="M15.5 1.5c-1.5 0-2.7.9-3.5 2.1-.7 1.2-1.1 2.7-.9 4.1 1.6.2 3.1-.6 4-1.8.8-1.2 1.2-2.7.4-4.4zM16.5 3.2c.4 1.3 0 2.6-.7 3.6-.6.9-1.8 2-3.3 2.1.1 1.7 1.3 3.4 2.5 4.3-.7.4-1.6.9-2.5.9-.9 0-1.6-.6-2.4-.6-.9 0-1.7.6-2.6.6-.9 0-1.8-.6-2.7-1.3-1.3-1-2.3-2.8-2.3-4.4 0-1.5.6-3 1.5-4 .8-.8 1.9-1.3 3-1.3.8 0 1.6.5 2.1.5.5 0 1.3-.6 2.4-.6 1.2 0 2.3.8 3 1.5zM15 16.8c.3-2 1-3.5 1.7-4.5.6-.9 1.5-1.6 2.4-1.9.5-.2 1.1-.3 1.4-.2-.8 1.9-1.9 3.8-3.1 5.3-.7.9-1.5 1.8-2.4 1.8-.9 0-1.5-.6-2.3-.6-.9 0-1.7.6-2.5.6-.8 0-1.5-.6-2-1.3-1.1-1.4-2-3.4-2-5.4 0-1.7.5-3.3 1.4-4.4.8-1 1.9-1.7 3.1-1.8 1.1-.1 2.1.5 2.9.5.7 0 1.8-.5 3-.5 1.1 0 2.1.7 2.8 1.6-1.7.8-3 2.6-3.2 5.2-.1 1.4.2 2.9.7 4z"/>"#
    } else if os.contains("darwin") || os.contains("mac") {
        r#"<path d="M15.5 1.5c-1.5 0-2.7.9-3.5 2.1-.7 1.2-1.1 2.7-.9 4.1 1.6.2 3.1-.6 4-1.8.8-1.2 1.2-2.7.4-4.4zM16.5 3.2c.4 1.3 0 2.6-.7 3.6-.6.9-1.8 2-3.3 2.1.1 1.7 1.3 3.4 2.5 4.3-.7.4-1.6.9-2.5.9-.9 0-1.6-.6-2.4-.6-.9 0-1.7.6-2.6.6-.9 0-1.8-.6-2.7-1.3-1.3-1-2.3-2.8-2.3-4.4 0-1.5.6-3 1.5-4 .8-.8 1.9-1.3 3-1.3.8 0 1.6.5 2.1.5.5 0 1.3-.6 2.4-.6 1.2 0 2.3.8 3 1.5zM15 16.8c.3-2 1-3.5 1.7-4.5.6-.9 1.5-1.6 2.4-1.9.5-.2 1.1-.3 1.4-.2-.8 1.9-1.9 3.8-3.1 5.3-.7.9-1.5 1.8-2.4 1.8-.9 0-1.5-.6-2.3-.6-.9 0-1.7.6-2.5.6-.8 0-1.5-.6-2-1.3-1.1-1.4-2-3.4-2-5.4 0-1.7.5-3.3 1.4-4.4.8-1 1.9-1.7 3.1-1.8 1.1-.1 2.1.5 2.9.5.7 0 1.8-.5 3-.5 1.1 0 2.1.7 2.8 1.6-1.7.8-3 2.6-3.2 5.2-.1 1.4.2 2.9.7 4z"/>"#
    } else {
        r#"<path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10 10-4.5 10-10S17.5 2 12 2zm0 18c-4.4 0-8-3.6-8-8s3.6-8 8-8 8 3.6 8 8-3.6 8-8 8zm1-8V7h-2v6h2zm0 4h-2v-2h2v2z"/>"#
    };
    format!(
        r#"<svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">{svg}</svg>"#
    )
}

fn relative_time(t: &str) -> String {
    if t.is_empty() || t.starts_with("0001") {
        return "未知".to_string();
    }
    let t = t.replace("Z", "+00:00");
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&t) {
        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
        if diff.num_seconds() < 60 {
            return "刚刚".to_string();
        }
        if diff.num_minutes() < 60 {
            return format!("{}分钟前", diff.num_minutes());
        }
        if diff.num_hours() < 24 {
            return format!("{}小时前", diff.num_hours());
        }
        if diff.num_days() < 7 {
            return format!("{}天前", diff.num_days());
        }
        return dt.format("%Y-%m-%d %H:%M").to_string();
    }
    t
}

fn get_devices() -> Vec<Value> {
    let Ok(status) = get_status() else {
        return vec![];
    };

    let mut devices = Vec::new();
    let self_info = &status["Self"];

    // 用户映射
    let mut user_map = std::collections::HashMap::new();
    if let Some(users) = status["User"].as_object() {
        for u in users.values() {
            if let Some(id) = u["ID"].as_i64() {
                user_map.insert(id, u["LoginName"].as_str().unwrap_or("").to_string());
            }
        }
    }

    // 本机
    let ips = self_info["TailscaleIPs"].as_array().cloned().unwrap_or_default();
    let ip = ips.first().map(|v| v.as_str().unwrap_or("")).unwrap_or("");
    let ipv6 = ips.get(1).map(|v| v.as_str().unwrap_or("")).unwrap_or("");
    devices.push(json!({
        "name": self_info["HostName"].as_str().unwrap_or(""),
        "icon": get_os_icon(self_info["OS"].as_str().unwrap_or("")),
        "ip": ip,
        "ipv6": ipv6,
        "online": true,
        "os": self_info["OS"].as_str().unwrap_or(""),
        "last_seen": "在线",
        "dns": self_info["DNSName"].as_str().unwrap_or("").trim_end_matches('.'),
        "user": user_map.get(&self_info["UserID"].as_i64().unwrap_or(0)).cloned().unwrap_or_default(),
        "self": true,
    }));

    // 其他设备
    if let Some(peers) = status["Peer"].as_object() {
        for p in peers.values() {
            let ips = p["TailscaleIPs"].as_array().cloned().unwrap_or_default();
            let ip = ips.first().map(|v| v.as_str().unwrap_or("")).unwrap_or("");
            let ipv6 = ips.get(1).map(|v| v.as_str().unwrap_or("")).unwrap_or("");
            let last_seen = p["LastSeen"].as_str().unwrap_or("");
            devices.push(json!({
                "name": p["HostName"].as_str().unwrap_or(""),
                "icon": get_os_icon(p["OS"].as_str().unwrap_or("")),
                "ip": ip,
                "ipv6": ipv6,
                "online": p["Online"].as_bool().unwrap_or(false),
                "os": p["OS"].as_str().unwrap_or(""),
                "last_seen": relative_time(last_seen),
                "dns": p["DNSName"].as_str().unwrap_or("").trim_end_matches('.'),
                "user": user_map.get(&p["UserID"].as_i64().unwrap_or(0)).cloned().unwrap_or_default(),
                "self": false,
            }));
        }
    }

    // 在线置顶
    devices.sort_by(|a, b| {
        let a_online = a["online"].as_bool().unwrap_or(false);
        let b_online = b["online"].as_bool().unwrap_or(false);
        b_online.cmp(&a_online)
    });

    devices
}

fn get_network_info() -> Value {
    let Ok(status) = get_status() else {
        return json!({});
    };

    let peers = status["Peer"].as_object().cloned().unwrap_or_default();
    let online_count = peers
        .values()
        .filter(|p| p["Online"].as_bool().unwrap_or(false))
        .count();
    let self_info = &status["Self"];

    json!({
        "total_devices": peers.len() + 1,
        "online": online_count + 1,
        "offline": peers.len() - online_count,
        "dns_name": self_info["DNSName"].as_str().unwrap_or("").trim_end_matches('.'),
        "version": self_info["Version"],
    })
}

fn ping_device(ip: &str) -> Value {
    let output = std::process::Command::new("ping")
        .args(["-c", "2", "-W", "2", ip])
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let re = regex::Regex::new(r"time[=<](\d+\.?\d*)").unwrap();
            if let Some(caps) = re.captures(&stdout) {
                return json!({"reachable": true, "latency_ms": caps[1].to_string()});
            }
            return json!({"reachable": true, "latency_ms": "?"});
        }
    }
    json!({"reachable": false, "latency_ms": null})
}

// ---------- HTTP 路由 ----------

async fn status_handler() -> impl IntoResponse {
    let devices = get_devices();
    let online = devices.iter().filter(|d| d["online"].as_bool().unwrap_or(false)).count();
    let offline = devices.len() - online;
    let status = get_status().unwrap_or(json!({}));
    let self_info = &status["Self"];
    let ips = self_info["TailscaleIPs"].as_array().cloned().unwrap_or_default();
    let ip = ips.first().map(|v| v.as_str().unwrap_or("")).unwrap_or("");

    Json(json!({
        "service": "正常运行",
        "devices": devices.len(),
        "online": online,
        "offline": offline,
        "hostname": self_info["HostName"].as_str().unwrap_or(""),
        "ip": ip,
    }))
}

async fn devices_handler() -> impl IntoResponse {
    Json(get_devices())
}

async fn network_handler() -> impl IntoResponse {
    Json(get_network_info())
}

async fn ping_handler(Path(ip): Path<String>) -> impl IntoResponse {
    Json(ping_device(&ip))
}

async fn health_handler() -> impl IntoResponse {
    match get_status() {
        Ok(status) => {
            let backend = status["BackendState"].as_str().unwrap_or("unknown");
            if backend == "Running" {
                (StatusCode::OK, Json(json!({"status": "ok", "tailscale_backend": backend})))
            } else {
                (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"status": "degraded", "tailscale_backend": backend})))
            }
        }
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"status": "error", "tailscale_backend": "unreachable", "error": e})),
        ),
    }
}

fn render_device_cards(devices: &[Value], online_only: bool) -> String {
    let mut html = String::new();
    for d in devices {
        let is_online = d["online"].as_bool().unwrap_or(false);
        if online_only != is_online {
            continue;
        }
        let name = d["name"].as_str().unwrap_or("");
        let icon = d["icon"].as_str().unwrap_or("");
        let os = d["os"].as_str().unwrap_or("");
        let dns = d["dns"].as_str().unwrap_or("");
        let ip = d["ip"].as_str().unwrap_or("");
        let ipv6 = d["ipv6"].as_str().unwrap_or("");
        let last_seen = d["last_seen"].as_str().unwrap_or("");
        let is_self = d["self"].as_bool().unwrap_or(false);
        let title_class = if is_online { "online" } else { "offline" };
        let badge_class = if is_online { "online-badge" } else { "offline-badge" };
        let status_text = if is_online {
            r#"<span data-i18n="online">在线</span>"#
        } else {
            r#"<span data-i18n="offline">离线</span>"#
        };

        let self_label = if is_self {
            r#"<span style="font-size:12px;color:#2563eb;" data-i18n="self-label">(本机)</span>"#
        } else {
            ""
        };

        let dns_row = if !dns.is_empty() {
            format!(r#"<div class="info"><span data-i18n="domain-label">域名</span>: {dns}</div>"#)
        } else {
            String::new()
        };
        let ipv6_row = if !ipv6.is_empty() {
            format!(r#"<div class="info ip" style="font-size:12px;" onclick="copyIP('{ipv6}')">IPv6: {ipv6}</div>"#)
        } else {
            String::new()
        };
        let last_row = if is_online {
            r#"<div class="info"><span data-i18n="status-label">状态</span>: <span data-i18n="online">在线</span></div>"#.to_string()
        } else {
            format!(r#"<div class="info"><span data-i18n="last-seen-label">最后在线</span>: {last_seen}</div>"#)
        };

        html.push_str(&format!(
            r#"<div class="card device">
    <div class="title {title_class}">
        {icon} {name} {self_label}
    </div>
    <div class="info"><span data-i18n="os-label">系统</span>: {os}</div>
    {dns_row}
    <div class="info ip" onclick="copyIP('{ip}')">IP: {ip}</div>
    {ipv6_row}
    {last_row}
    <button class="ping-btn" onclick="pingDevice('{ip}', this)"><svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor" style="vertical-align:-2px;margin-right:4px;"><path d="M12 7a5 5 0 0 0-5 5h2a3 3 0 0 1 6 0h2a5 5 0 0 0-5-5zm0 4a1 1 0 0 0-1 1v3a1 1 0 0 0 2 0v-3a1 1 0 0 0-1-1zM5.6 9.6A8 8 0 0 0 4 14h2a6 6 0 0 1 12 0h2a8 8 0 0 0-14.4-4.4zM12 2a12 12 0 0 0-8.5 3.6l1.4 1.4A10 10 0 0 1 12 4a10 10 0 0 1 7.1 3l1.4-1.4A12 12 0 0 0 12 2z"/></svg><span data-i18n="ping-btn">测试连通性</span></button>
    <div class="ping-result"></div>
    <div class="badge {badge_class}">{status_text}</div>
</div>
"#,
        ));
    }
    html
}

async fn panel_handler() -> impl IntoResponse {
    let devices = get_devices();
    let online = devices.iter().filter(|d| d["online"].as_bool().unwrap_or(false)).count();
    let offline = devices.len() - online;
    let net = get_network_info();

    let online_cards = render_device_cards(&devices, true);
    let offline_cards = render_device_cards(&devices, false);

    let html = include_str!("../templates/panel.html");
    let mut html = html
        .replace("{{ net.total_devices }}", &net["total_devices"].to_string())
        .replace("{{ net.online }}", &net["online"].to_string())
        .replace("{{ net.offline }}", &net["offline"].to_string())
        .to_string();

    // 替换整个 Jinja 循环块 (从 {% for ... %} 到 {% endif %}{% endfor %})
    html = replace_jinja_block(&html, r#"{% for d in devices %}{% if d.online %}"#, r#"{% endif %}{% endfor %}"#, &online_cards);
    html = replace_jinja_block(&html, r#"{% for d in devices %}{% if not d.online %}"#, r#"{% endif %}{% endfor %}"#, &offline_cards);

    Html(html)
}

/// 替换两个标记之间的完整 Jinja 块为渲染内容
fn replace_jinja_block(html: &str, start_marker: &str, end_marker: &str, replacement: &str) -> String {
    if let Some(start_idx) = html.find(start_marker) {
        if let Some(rel_end) = html[start_idx..].find(end_marker) {
            let end_idx = start_idx + rel_end + end_marker.len();
            let mut result = String::with_capacity(html.len() + replacement.len());
            result.push_str(&html[..start_idx]);
            result.push_str(replacement);
            result.push_str(&html[end_idx..]);
            return result;
        }
    }
    html.to_string()
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(status_handler))
        .route("/devices", get(devices_handler))
        .route("/network", get(network_handler))
        .route("/ping/:ip", get(ping_handler))
        .route("/health", get(health_handler))
        .route("/panel", get(panel_handler))
        .nest_service("/static", tower_http::services::ServeDir::new("static"));

    let port: u16 = env_or("PORT", &PORT.to_string()).parse().unwrap_or(PORT);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.unwrap();
    println!("Tailscale 面板启动于 :{port}");
    axum::serve(listener, app).await.unwrap();
}
