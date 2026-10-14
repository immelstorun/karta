# Changelog

Все значимые изменения проекта karta документируются здесь.

Формат основан на [Keep a Changelog](https://keepachangelog.com/ru/1.1.0/),
версионирование — [SemVer](https://semver.org/lang/ru/).

## [0.0.1] — 2026-10-14

### Added

- **Cargo workspace** с 8 крейтами:
  - `karta-core` — гео-математика (LatLon, TileId, WebMercator), тайловая математика (XYZ/TMS/QuadKey), векторная модель (Track/Waypoint/Route)
  - `karta-net` — HTTP-клиент (reqwest + rustls), tile fetcher с retry и exponential backoff, token bucket rate limiter
  - `karta-cache` — SAS.Planet-совместимый кэш тайлов (структура `z{zoom+1}/x{N}/y{N}.ext`, `.tne` маркеры), MBTiles (SQLite), in-memory LRU
  - `karta-formats` — GPX 1.1 парсинг/экспорт (quick-xml), GeoJSON парсинг/экспорт, MBTiles metadata, заглушки KML/JNX/GeoTIFF
  - `karta-render` — заглушка рендерера (wgpu)
  - `karta-plugin-host` — WASM-плагины (wasmtime), permissions, декларативные TOML-слои
  - `karta-route-db` — база маршрутов, модель данных
  - `karta-ai-agents` — AI-агенты: toponym extractor, route thread builder
- **Desktop-приложение** (`apps/desktop`):
  - Интерактивная карта с OSM-тайлами (реальная загрузка по HTTP)
  - Pan (drag), zoom (scroll + zoom к точке под курсором)
  - Кэширование в SAS.Planet-совместимом формате
  - Статусбар с координатами курсора
  - Боковая панель с информацией
- **CLI-утилиты** (`apps/cli`):
  - `karta-cli download` — скачивание тайлов области
  - `karta-cli cache` — статистика кэша
  - `karta-cli layers` — список доступных слоёв
- **80 юнит-тестов** — все проходят

### Known Issues

- KML/JNX/GeoTIFF — заглушки (не реализованы)
- Android/iOS/Web — не реализованы (в roadmap)
- Plugin API — заглушка (wasmtime подключён, host functions не экспортированы)
