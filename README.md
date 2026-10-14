# karta

> Кроссплатформенная картографическая платформа для туристов.
> Мощность SAS.Planet + удобство nakarte.me + модульность Factorio.

## Зачем

Существующие инструменты для работы с картами в походах либо устарели, либо ограничены:

- **SAS.Planet** — мощный движок скачивания тайлов, кэширования и экспорта, но Windows-only, UI из 2008 года, 19-летний легаси-код на Delphi, COM-интерфейсный суп на 1600+ юнитов.
- **nakarte.me** — лёгкий веб-просмотр карт с треками, но без оффлайн-режима, без скачивания областей, без мобильного приложения, монолитный фронтенд на Knockout.js.
- **OsmAnd / Organic Maps** — мобильные навигаторы, но без desktop-версии, без мощного экспорта, с закрытой архитектурой плагинов.

**karta** объединяет лучшее из этих миров:

| Возможность | karta |
|---|---|
| Платформы | Windows, Linux, macOS, Android, Web |
| Оффлайн | Полноценный (SAS.Planet-совместимый кэш) |
| Скачивание областей | Да (по полигону, с выбором зумов) |
| Треки/метки | Да (GPX/KML/GeoJSON) |
| Плагины | WASM + декларативные TOML-слои |
| Хранение тайлов | Локальный кэш + BitTorrent-дистрибуция пакетов |
| AI-маршруты | Интеграция с базой туристских маршрутов (tlib.ru) |
| Ядро | Rust (компилируется в native + WASM) |

## Философия

### 1. Factorio-подход к архитектуре

Factorio известен тем, что при колоссальной сложности остаётся ультраоптимизированным, лёгким и модульным. karta следует тем же принципам:

- **Маленький бинарник** — минимум зависимостей, никаких рантаймов
- **Быстрый старт** — ленивая инициализация, отложенная загрузка
- **Модульность** — каждый компонент — отдельный крейт, отключаемый на этапе сборки
- **Стабильный Plugin API** — версионируемый интерфейс для сторонних расширений

### 2. Локальность прежде всего

Данные пользователя (метки, треки, кэш) хранятся локально. Облачная синхронизация — опция, не требование.

### 3. Сообщество как движок

- Плагины от сообщества через WASM-песочницу
- Распределённое хранение тайлов через BitTorrent
- Community-база маршрутов с валидацией человеком

## Архитектура

```
┌─────────────────────────────────────────────────────────┐
│  Слой 3: Platform Shells (тонкие обёртки)                │
│  Desktop(Win/Linux/Mac) │ Android │ iOS │ Web            │
├─────────────────────────────────────────────────────────┤
│  Слой 2: Service Layer (self-hosted, опционально)        │
│  Tile Server (MBTiles+nginx) │ Elevation │ Route DB      │
│  CORS-proxy │ Geocoding │ AI Agents (tlib.ru parsing)    │
├─────────────────────────────────────────────────────────┤
│  Слой 1: Rust Core Engine (единое ядро)                  │
│  • tile-math + projections (без зависимостей)            │
│  • tile fetching + caching (SAS-cache совместимость)     │
│  • vector data model (tracks, marks, routes)             │
│  • format I/O (GPX/KML/GeoJSON/MBTiles/JNX/GeoTIFF)      │
│  • rendering pipeline (wgpu)                             │
│  • plugin host (WASM runtime, permissions, registry)     │
│  Компилируется в: native binary + WASM (для web)         │
└─────────────────────────────────────────────────────────┘
```

### Крейты

| Крейт | Назначение |
|---|---|
| `karta-core` | Гео-математика, тайлы, проекции, векторная модель |
| `karta-net` | HTTP-клиент, загрузка тайлов с retry, rate limiting |
| `karta-cache` | Кэш тайлов: SAS.Planet-совместимый, MBTiles, in-memory LRU |
| `karta-formats` | Импорт/экспорт: GPX, KML, GeoJSON, MBTiles, JNX, GeoTIFF |
| `karta-render` | Рендеринг карты: wgpu-based tile and vector rendering |
| `karta-plugin-host` | Плагинная система: WASM runtime, permissions, registry |
| `karta-route-db` | База маршрутов: tlib.ru integration, community routes |
| `karta-ai-agents` | AI-агенты для построения маршрутов из текстовых описаний |

### Приложения

| Приложение | Платформы |
|---|---|
| `apps/desktop` | Windows, Linux, macOS (egui/eframe) |
| `apps/cli` | CLI-утилиты: кэш-менеджмент, скачивание, конвертация |

## Плагинная система

karta поддерживает три уровня расширений:

### Уровень 1: Декларативные слои (TOML, без кода)

```toml
[layer]
id = "osm-standard"
name = "OpenStreetMap Standard"
url_template = "https://tile.openstreetmap.org/{z}/{x}/{y}.png"
min_zoom = 0
max_zoom = 19
attribution = "© OpenStreetMap contributors"
```

### Уровень 2: WASM-плагины (полноценная логика)

Плагин можно писать на любом языке, компилирующемся в WASM — Rust, Go, TypeScript/AssemblyScript, C/C++, Zig.

```toml
[plugin]
id = "com.example.strava-heatmap"
name = "Strava Heatmap"
version = "1.2.0"
api-version = "1"

[permissions]
network = ["*.strava.com"]
storage = ["plugin-data"]
```

### Уровень 3: AI-агенты (для построения маршрутов)

- Парсинг текста отчёта → извлечение топонимов
- Геокодирование → координаты
- Построение нитки маршрута
- Валидация человеком

## Распределённое хранение тайлов

### Статичные датасеты → BitTorrent

Советские топокарты (ГГЦ, Topo, Arbalet), исторические карты, карты гор, elevation-данные — **иммутабельны**. BitTorrent-дистрибуция через сообщество:

- Сервер отдаёт magnet-ссылку, сиды = сообщество
- Экономия: вместо 1 ТБ хостинга = только трекер (легковесный)

### Динамические данные → локальный кэш

Google/Yandex/Bing/OSM свежие слои:
- Приложение качает напрямую с провайдера, кэширует у себя (SAS-cache / MBTiles)
- Peer-assisted модель: каждый юзер строит свой локальный кэш

## Дорожная карта

### Phase 0: Core Foundation ✅ (текущая, 0.0.1)
- [x] Cargo workspace + все крейты-заглушки
- [x] Базовая гео-математика (tiles, projections, vector)
- [x] HTTP-клиент + tile fetching с retry
- [x] SAS.Planet-совместимый кэш
- [x] GPX парсинг/экспорт, GeoJSON парсинг/экспорт
- [x] Desktop-приложение с OSM-картой (pan/zoom/caching)
- [x] CLI-утилиты (download, cache stats)
- [x] Плагинная система: WASM runtime + permissions + TOML layers
- [x] AI-агенты: toponym extractor + route thread builder (заглушки)

### Phase 1: MVP Map Viewer (1–2 месяца)
- [ ] Поддержка слоёв (OSM, Google, Yandex, Bing, ESRI)
- [ ] Оверлеи (топо-карты поверх спутника)
- [ ] Полноценный SAS-cache reader/writer
- [ ] Загрузка областей по полигону (Region Download)
- [ ] Экспорт GeoTIFF / KMZ / MBTiles
- [ ] Метки/категории (MarksDB)
- [ ] Импорт GPX / KML / GeoJSON

### Phase 2: Plugin System (1–2 месяца)
- [ ] WASM host functions (karta API exposed to plugins)
- [ ] Declarative TOML layer definitions
- [ ] Plugin permissions system
- [ ] Plugin registry (central repo)
- [ ] Примеры плагинов (Strava heatmap, custom layers)

### Phase 3: Torrent Distribution (1–2 месяца)
- [ ] BitTorrent-клиент (librqbit или аналог)
- [ ] Magnet-link registry для статичных пакетов
- [ ] MBTiles packs: скачивание → офлайн-просмотр
- [ ] Обновление пакетов через versioned magnet links

### Phase 4: Region Download & Export (2–3 месяца)
- [ ] Скачивание областей (по полигону/прямоугольнику) с выбором зумов
- [ ] Экспорт в GeoTIFF, KMZ, MBTiles, JNX, OruxMaps, RMaps
- [ ] Прогресс-бар, отмена, лимиты
- [ ] Сшивка больших изображений (Combine)

### Phase 5: Mobile Platforms (2–3 месяца)
- [ ] Android (Tauri 2 или egui + android-activity)
- [ ] Touch-жесты (pinch-zoom, pan, long-tap)
- [ ] Оффлайн-режим (кэш уже работает)
- [ ] iOS (если будет спрос)

### Phase 6: Route Database & AI Agents (2–3 месяца)
- [ ] Парсинг отчётов tlib.ru → извлечение текстов ниток маршрутов
- [ ] Геокодер топонимов (перевалы, реки, избы) → координаты
- [ ] AI-построение нитки маршрута (снаппинг к тропам через OSM)
- [ ] Валидация человеком → правки сохраняются в базу
- [ ] Community-база маршрутов с рейтингами

### Phase 7: Community & Sync (2–3 месяца)
- [ ] Аккаунты (опционально, локальный режим всегда работает)
- [ ] Синхронизация меток/треков между устройствами
- [ ] Публичные треки / маршруты с permalink
- [ ] Elevation Server (SRTM) self-hosted

## Участие в разработке

Мы приветствуем вклад в любом виде:

- **Баг-репорты** — создайте issue
- **Feature requests** — обсудите в discussions
- **Плагины** — напишите WASM-плагин для нового слоя/функции
- **Датасеты** — станьте сидом для статичных тайл-пакетов

## Лицензия

GPL-3.0-or-later. См. [LICENSE](LICENSE).

## Благодарности

- [SAS.Planet](https://github.com/sasgis/sas.planet.src) — вдохновение и 19 лет опыта работы с тайлами
- [nakarte.me](https://github.com/wladich/nakarte) — UX-референс и открытые тайлсеты
- [tlib.ru](https://tlib.ru) — крупнейшая библиотека отчётов о походах
