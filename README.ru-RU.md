# Tailscale Панель

[English](README.md) | [简体中文](README.zh-CN.md) | [Русский](README.ru-RU.md)

Лёгкая веб-панель для мониторинга статуса узлов сети Tailscale. Доступна с любого устройства в вашей локальной сети (телефон, планшет, компьютер).

## Возможности

- 📊 **Список устройств**: статус онлайн/офлайн, тип ОС, IPv4/IPv6, домен MagicDNS, владелец
- 🔍 **Поиск и копирование**: поиск устройств, клик по IP для копирования
- 📡 **Тест соединения**: проверка связи с любым устройством в один клик, показывает задержку
- 🌓 **Режимы темы**: авто (по времени суток) / как в системе / светлая / тёмная
- 🔄 **Автообновление**: каждые 30 секунд
- 🖼️ **Свой фон**: загрузка изображения, настройка прозрачности, полупрозрачные карточки

## Скриншот

![Панель Tailscale устройств](docs/screenshot.png)

## Технологии

- **Бэкенд на Rust** (фреймворк Axum) — компилируется в один статический бинарник, без зависимостей
- **Мультиархитектурность**: готовые сборки `linux/amd64` и `linux/arm64`
- **Минимальный образ**: всего ~19MB, один статический бинарник
- Общение с `tailscaled` через локальный socket API (бинарник tailscale не нужен)

## Быстрый старт

### Вариант 1: Готовый образ (рекомендуется)

Без локальной сборки (мультиархитектурный: amd64 + arm64):

**Шаг 1: Загрузите образ**

```bash
docker pull ghcr.io/saves24/tailscale-api:latest
```

**Шаг 2: Запустите контейнер**

```bash
docker run -d --name tailscale-api \
  --network host \
  -v /var/run/tailscale:/var/run/tailscale \
  --restart unless-stopped \
  ghcr.io/saves24/tailscale-api:latest
```

> Использует режим сети `host` (чтение статистики сети хоста). Доступ: `http://<хост>:8091/panel`

### Вариант 2: Сборка из исходников

```bash
# arm64 (на Pi):
docker build --build-arg BINARY=tailscale-arm64 -t tailscale-api .
# amd64 (на x86):
docker build --build-arg BINARY=tailscale-amd64 -t tailscale-api .

# Или через compose:
docker compose up -d
```

Затем откройте: `http://<хост>:8091/panel`

## Требования

- Docker + Docker Compose
- Tailscale запущен на хосте (tailscaled)
- Связь с tailscaled через смонтированный сокет `/var/run/tailscale`

## Конфигурация

| Переменная | По умолчанию | Описание |
|---|---|---|
| `CACHE_TTL` | `5` | Время кэша (сек) для `tailscale status` |

## API

| Эндпоинт | Описание |
|---|---|
| `GET /` | Статус сервиса (JSON) |
| `GET /devices` | Список устройств (JSON) |
| `GET /network` | Статистика сети (JSON) |
| `GET /ping/<ip>` | Тест соединения (JSON) |
| `GET /panel` | Веб-панель |

## Структура проекта

```
├── src/main.rs          # Приложение Rust (Axum)
├── Cargo.toml           # Зависимости Rust
├── templates/panel.html # HTML-шаблон (встраивается при компиляции)
├── static/              # CSS / JS (читаются в рантайме)
├── Dockerfile           # Сборка контейнера (ARG BINARY для архитектуры)
├── docker-compose.yml   # Конфигурация Compose
└── .gitignore
```

## Лицензия

MIT
