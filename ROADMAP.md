# ROADMAP

> Мегадетальный роадмап проекта karta — кроссплатформенной картографической платформы для туристов.
> Цель: объединить мощность SAS.Planet с удобством nakarte.me, добавить модульность Factorio.

## Обзор фаз

| Фаза | Название | Длительность | Статус |
|---|---|---|---|
| 0 | Core Foundation | 2 недели | ✅ Завершена (0.0.1) |
| 1 | MVP Map Viewer | 1–2 месяца | 🔜 Следующая |
| 2 | Plugin System | 1–2 месяца | 📋 Запланирована |
| 3 | Torrent Distribution | 1–2 месяца | 📋 Запланирована |
| 4 | Region Download & Export | 2–3 месяца | 📋 Запланирована |
| 5 | Mobile Platforms | 2–3 месяца | 📋 Запланирована |
| 6 | Route Database & AI Agents | 2–3 месяца | 📋 Запланирована |
| 7 | Community & Sync | 2–3 месяца | 📋 Запланирована |

---

## Phase 0: Core Foundation ✅

**Цель:** Создать каркас проекта — все крейты как заглушки, компилируемые и тестируемые.

### Что сделано

- [x] Cargo workspace с версионированием из workspace
- [x] `karta-core`:
  - [x] `LatLon` — координаты с валидацией
  - [x] `TileId` — идентификатор тайла (x, y, zoom)
  - [x] `lat_lon_to_tile()` / `tile_to_lat_lon()` — преобразования
  - [x] `quadkey()` — Bing/Google quadkey
  - [x] `WebMercator` — проекция (meters ↔ lat/lon, resolution)
  - [x] `Track` / `TrackPoint` / `Waypoint` / `Route` — векторная модель
  - [x] `haversine_distance()` — расстояние между точками
  - [x] `Track::length_meters()` — длина трека
- [x] `karta-net`:
  - [x] `HttpClient` — reqwest + rustls, user-agent, timeout
  - [x] `RateLimiter` — token bucket
  - [x] `TileFetcher` — retry с exponential backoff
- [x] `karta-cache`:
  - [x] `SasCache` — SAS.Planet-совместимый формат (`z{zoom+1}/x{N}/y{N}.ext`)
  - [x] `.tne` маркеры ("тайл не существует")
  - [x] `MbtilesCache` — MBTiles SQLite (с конверсией XYZ↔TMS y)
  - [x] `MemoryCache` — LRU кэш
- [x] `karta-formats`:
  - [x] `parse_gpx()` / `export_gpx()` — GPX 1.1
  - [x] `parse_geojson()` / `export_geojson()` — GeoJSON
  - [x] Заглушки: KML, JNX, GeoTIFF
- [x] `karta-render` (заглушка):
  - [x] `RenderConfig`
  - [x] `TileRenderer` — заглушка wgpu pipeline
  - [x] `LayerCompositor`
- [x] `karta-plugin-host` (заглушка):
  - [x] WASM runtime (wasmtime)
  - [x] `Permissions` — система разрешений (glob-matching для network/fs/ui)
  - [x] `LayerManifest` — TOML-манифест слоя
  - [x] `build_tile_url()` — построение URL из LayerSpec
- [x] `karta-route-db` (заглушка):
  - [x] `Route` — модель маршрута
  - [x] `TlibClient` — клиент tlib.ru (заглушка HTTP)
- [x] `karta-ai-agents` (заглушка):
  - [x] `ToponymExtractor` — извлечение топонимов
  - [x] `RouteThreadBuilder` — построение нитки маршрута
  - [x] `RouteValidator` — валидация маршрута
- [x] Desktop-приложение (`apps/desktop`):
  - [x] egui/eframe
  - [x] Реальная загрузка OSM-тайлов
  - [x] Pan/zoom/caching
- [x] CLI (`apps/cli`):
  - [x] `download` — скачивание области
  - [x] `cache` — статистика кэша
  - [x] `layers` — список слоёв
- [x] 80 юнит-тестов

---

## Phase 1: nakarte.me Tile Proxy 🔜

**Цель:** Проксировать тайлы nakarte.me в UI karta — быстрый путь к рабочей карте с топослоями.

**Приоритет:** ПЕРВЫЙ — вместо самостоятельной инфраструктуры тайлов используем готовые тайлы nakarte.me.

### 1.1 Слои nakarte.me

- [x] Исследование доступных слоёв nakarte.me
- [x] TOML-манифесты для слоёв nakarte.me (ggc2000, ggc1000, ggc500, ggc250, topo1000, topo500, topo250, new_gsh_100k, new_gsh_050k, purikov, topomapper)
- [x] Поддержка TMS Y-flip в URL-шаблонах (`tms_y_flip = true`)
- [ ] Поддержка subdomains (`{s}` → a/b/c) для распределения нагрузки
- [x] Автоматическая конвертация XYZ → TMS в `TileFetcher`

### 1.2 Загрузка и кэширование

- [x] Интеграция `karta-net` + `karta-cache` с nakarte.me слоями
- [x] Кэширование в SAS.Planet-совместимом формате
- [x] `.tne` маркеры для несуществующих тайлов (404 → маркер в кэше)
- [ ] LRU eviction для GPU-текстур

### 1.3 UI

- [x] Панель слоёв с переключением между слоями nakarte.me
- [ ] Настройка прозрачности слоёв (для оверлеев)
- [ ] Отображение attribution
- [ ] Статусбар: zoom, координаты, источник тайлов

### Milestone: v0.1.0 — "nakarte.me Proxy"

Критерии готовности:
- Загрузка и отображение тайлов nakarte.me (ggc500, topomapper) в desktop UI
- Pan/zoom с кэшированием
- TMS Y-flip работает корректно
- Переключение между слоями

---

## Phase 2: Plugin System 📋

**Цель:** Плагинная система с WASM-песочницей и декларативными TOML-слоями.

### 2.1 Декларативные слои (TOML)

- [ ] Формат TOML-манифеста слоя:
  - `id`, `name`, `description`
  - `url_template` с подстановками `{z}/{x}/{y}/{s}/{q}/{-y}`
  - `min_zoom`, `max_zoom`
  - `subdomains` (для `{s}`)
  - `attribution`
  - `tile_size` (256/512)
  - `tms_y_flip` (TMS vs XYZ)
  - `bounds` (ограничение слоя по географическим координатам)
  - `headers` (custom HTTP headers, например Referer)
  - `cache_type` (sas-planet / mbtiles / memory)
- [ ] Парсер TOML → `LayerSpec`
- [ ] Загрузка слоёв из каталога `~/.config/karta/layers/`
- [ ] Примеры декларативных слоёв (OSM, ESRI, OpenTopoMap, CyclOSM)

### 2.2 WASM-плагины

- [ ] `WasmRuntime` — инициализация wasmtime Engine
- [ ] Host functions для плагинов:
  - `karta.http_get(url) -> bytes` (только если permission `network` разрешает)
  - `karta.log(level, message)`
  - `karta.register_layer(spec)`
  - `karta.get_config(key)`
- [ ] `PluginInstance` — загрузка .wasm модуля через wasmtime
- [ ] Система permissions:
  - `network: ["*.openstreetmap.org"]` — доступ только к разрешённым доменам
  - `storage: ["plugin-data"]` — доступ только к своей папке
  - `ui: ["layers-panel"]` — доступ только к разрешённым UI-элементам
- [ ] Загрузка плагинов из `~/.config/karta/plugins/`
- [ ] Пример WASM-плагина (Rust → WASM)

### 2.3 Plugin Registry

- [ ] Центральный реестр плагинов (self-hosted)
- [ ] Каталог плагинов: поиск, категории, рейтинги, скриншоты
- [ ] Установка из UI в один клик (скачивание .wasm + манифест)
- [ ] Автообновление плагинов
- [ ] Подписи плагинов (цифровая подпись автора)
- [ ] Версионирование Plugin API (semver)

### Milestone: v0.2.0 — "Plugins"

Критерии готовности:
- Декларативные TOML-слои загружаются из конфига
- WASM-плагины работают в песочнице (wasmtime)
- Permissions ограничивают доступ плагинов к сети/файлам/UI
- Plugin registry (self-hosted) доступен
- Документация Plugin API v1

---

## Phase 3: Torrent Distribution 📋

**Цель:** Распределённое хранение статичных датасетов через BitTorrent.

### 3.1 BitTorrent-клиент

- [ ] Интеграция `librqbit` (или реализация на основе `tokio`)
- [ ] Поддержка magnet-ссылок
- [ ] Прогресс загрузки / отдачи (upload)
- [ ] Настройка bandwidth limits (download/upload)
- [ ] DHT (Distributed Hash Table) для поиска пиров без центрального трекера
- [ ] Peer Exchange (PEX) — обмен списками пиров
- [ ] Локальный peer discovery (LAN)

### 3.2 Magnet-Registry для статичных пакетов

- [ ] Реестр датасетов: magnet-ссылка + версия + размер + SHA-256
- [ ] Каталог доступных датасетов (в UI)
- [ ] Скачивание пакета → оффлайн-просмотр без сервера
- [ ] Versioned magnet links: при обновлении датасета (v2025 → v2026) — качается новая версия
- [ ] Автоматическое сидирование скачанных датасетов

### 3.3 MBTiles Packs

- [ ] Поддержка MBTiles как источника тайлов для движка рендеринга
- [ ] UI: менеджер датасетов (скачанные / доступные / обновления)
- [ ] Примеры датасетов (для тестирования):
  - Советские топокарты (ГГЦ)
  - Карты гор (Slazav)
  - Elevation data (SRTM)

### Milestone: v0.3.0 — "Torrents"

Критерии готовности:
- BitTorrent-клиент интегрирован (librqbit)
- Magnet-registry для статичных пакетов работает
- Датасеты скачиваются и отображаются офлайн
- Обновление пакетов через versioned magnet links

---

## Phase 4: Region Download & Export 📋

**Цель:** Скачивание областей карты и экспорт в навигационные форматы.

### 4.1 Region Download (по полигону/прямоугольнику)

- [ ] UI: выбор области (прямоугольник / полигон)
- [ ] Выбор уровней зума (например, z10–z14)
- [ ] Подсчёт количества тайлов до начала скачивания
- [ ] Прогресс-бар с отменой
- [ ] Ограничения: rate limiting, максимальное количество тайлов
- [ ] Фоновое скачивание (не блокирует UI)
- [ ] Retry при ошибках сети

### 4.2 Экспорт в форматы

- [ ] **GeoTIFF** — экспорт области с геопривязкой (через `tiff` crate + GeoKeys)
- [ ] **KMZ** — экспорт для Google Earth (zip + KML + tiles)
- [ ] **MBTiles** — экспорт в MBTiles SQLite (для MobAC / Locus)
- [ ] **JNX** — экспорт для Garmin BirdsEye (бинарный формат)
- [ ] **OruxMaps** — экспорт для OruxMaps (SQLite)
- [ ] **RMaps** — экспорт для RMaps (SQLite)
- [ ] **SAS4Android/CE** — экспорт для SAS.Planet mobile
- [ ] **Tar/Zip** — экспорт в архив

### 4.3 Сшивка больших изображений (Combine)

- [ ] Сшивка области в один большой файл (BMP/JPG/PNG/RAW)
- [ ] Настройка выходного разрешения
- [ ] Композиция слоёв (например, спутник + топо-оверлей)

### Milestone: v0.4.0 — "Region Download & Export"

Критерии готовности:
- Выбор области → скачивание → кэш
- Экспорт GeoTIFF / KMZ / MBTiles / JNX / OruxMaps / RMaps
- Прогресс-бар, отмена, лимиты
- CLI: `karta-cli download-region`, `karta-cli export`

---

## Phase 5: Mobile Platforms 📋

**Цель:** Запуск karta на Android и iOS.

### 5.1 Android

- [ ] Настройка Android-проекта (Gradle + NDK)
- [ ] Компиляция Rust-ядра в `libkarta.so` (aarch64-linux-android)
- [ ] UI: адаптация egui-приложения под touch-интерфейс
- [ ] Жесты: pinch-zoom, two-finger pan, long-tap
- [ ] Оффлайн-режим: кэш работает локально
- [ ] Интеграция с Android Intent system (share location)
- [ ] Google Play / F-Droid публикация

### 5.2 iOS

- [ ] Настройка iOS-проекта (Xcode + Rust)
- [ ] Компиляция для aarch64-apple-ios
- [ ] App Store публикация (если пройдёт ревью)

### Milestone: v0.5.0 — "Mobile"

Критерии готовности:
- Android APK собирается через CI
- Touch-жесты работают (pinch-zoom, pan)
- Оффлайн-режим работает
- Базовый набор фич (просмотр карты, кэш, слои) работает на мобильных

---

## Phase 6: Route Database & AI Agents 📋

**Цель:** Интеграция с базой туристских маршрутов и AI-построение маршрутов.

### 6.1 Парсинг tlib.ru

- [ ] HTTP-клиент для tlib.ru (выгрузка отчётов)
- [ ] Парсинг PDF → текст (или HTML → текст)
- [ ] Извлечение текста нитки маршрута (раздел "Ход путешествия")
- [ ] Кэширование спарсенных данных

### 6.2 AI-построение нитки маршрута

- [ ] Извлечение топонимов (перевалы, реки, вершины, избы) из текста
- [ ] Геокодирование топонимов:
  - Внутренняя база топонимов (Westra passes, OSM)
  - LLM-интеграция для сложных случаев
- [ ] Построение нитки маршрута (соединение точек)
- [ ] Снаппинг к тропам через OSM-данные (optional)
- [ ] Валидация человеком → правки сохраняются в базу

### 6.3 Community-база маршрутов

- [ ] Публичные маршруты с permalink
- [ ] Рейтинги / комментарии
- [ ] Теги (регион, сложность, сезон)
- [ ] Поиск по базе (по региону, названию, тегам)

### Milestone: v0.6.0 — "Routes & AI"

Критерии готовности:
- Парсинг tlib.ru → база маршрутов
- AI-построение нитки маршрута из текста отчёта
- Валидация человеком → сохранение в базу
- Community-база маршрутов с поиском

---

## Phase 7: Community & Sync 📋

**Цель:** Сообщество, синхронизация, распределённое хранение.

### 7.1 Аккаунты и синхронизация

- [ ] Опциональные аккаунты (локальный режим всегда работает)
- [ ] Синхронизация меток/треков между устройствами
- [ ] Экспорт/импорт всех данных (JSON/ZIP)
- [ ] E2E-шифрование синхронизируемых данных

### 7.2 Публичные маршруты / треки

- [ ] Публикация трека → permalink
- [ ] Встраивание карты на сторонний сайт (iframe)
- [ ] API для внешних сервисов

### 7.3 Elevation Server

- [ ] Self-hosted сервер высот (SRTM-based)
- [ ] Профиль высот по треку
- [ ] Отображение высот на карте (heatmap)

### 7.4 Интеграции

- [ ] Strava (импорт треков)
- [ ] Garmin Connect
- [ ] Wikiloc (импорт треков)
- [ ] GitHub (синхронизация настроек)

### Milestone: v0.7.0 — "Community & Sync"

Критерии готовности:
- Опциональная синхронизация между устройствами
- Публичные треки с permalink
- Elevation Server self-hosted
- Интеграции со Strava / Garmin / Wikiloc

---

## Технический долг и приоритеты

### Критические зависимости

| Крейт | Статус | Приоритет |
|---|---|---|
| `librqbit` (torrent) | Заглушка | Высокий (Phase 3) |
| `wasmtime` (plugins) | Подключён | Высокий (Phase 2) |
| `wgpu` (rendering) | Заглушка | Средний (Phase 1–2) |
| `tiff` (GeoTIFF) | Заглушка | Средний (Phase 4) |
| `rusqlite` (SQLite) | Подключён | Высокий (Phase 1) |

### Риски

| Риск | Вероятность | Митигация |
|---|---|---|
| Сложность WASM-плагинов (sandboxing, permissions) | Средняя | Начать с декларативных TOML-слоёв |
| Android/iOS сборка сложна | Высокая | Начать с Android (больше пользователей), iOS отложить |
| tlib.ru не даёт API | Средняя | Парсинг HTML/PDF через self-hosted сервис |
| AI-построение маршрутов неточно | Высокая | Валидация человеком обязательна |
| Сообщество не примет платформу | Средняя | Совместимость с SAS.Planet кэшем и nakarte.me тайлами как bridge |

---

## Вклад в разработку

См. [README.md](README.md) для описания архитектуры и [CHANGELOG.md](CHANGELOG.md) для истории изменений.
