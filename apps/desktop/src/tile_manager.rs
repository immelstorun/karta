//! Менеджер тайлов: async-загрузка, SAS.Planet кэш, память.

use crate::url_builder::UrlTemplate;
use image::DynamicImage;
use karta_cache::traits::TileCache;
use karta_cache::SasCache;
use karta_core::tiles::TileId;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Сообщение от фоновой задачи загрузки тайла.
pub enum TileResponse {
    /// Тайл успешно загружен (сеть или кэш) и декодирован.
    Loaded(TileId, DynamicImage),
    /// Ошибка загрузки тайла.
    Error(TileId, String),
}

/// Имя слоя OSM в SAS.Planet кэше.
const LAYER: &str = "osm";

/// Максимальное количество тайлов в памяти.
const MAX_MEMORY_TILES: usize = 500;

/// Менеджер тайлов: скрывает за собой кэш (SAS.Planet), async-загрузку
/// по HTTP и память загруженных тайлов.
pub struct TileManager {
    /// Handle на tokio runtime (общий для всех загрузок).
    runtime: Arc<tokio::runtime::Runtime>,
    /// HTTP-клиент для загрузки тайлов.
    client: reqwest::Client,
    /// SAS.Planet-совместимый файловый кэш.
    sas_cache: Arc<SasCache>,
    /// Шаблон URL тайлового сервера.
    url_template: UrlTemplate,
    /// Тайлы в процессе загрузки: TileId → время начала загрузки.
    pending_tiles: HashMap<TileId, Instant>,
    /// Загруженные тайлы (декодированные изображения).
    loaded_tiles: HashMap<TileId, DynamicImage>,
    /// Порядок загрузки тайлов (для LRU eviction).
    tile_order: Vec<TileId>,
    /// Канал для отправки результатов загрузки в UI-поток.
    sender: Sender<TileResponse>,
    /// Канал для приёма результатов загрузки в UI-потоке.
    receiver: Receiver<TileResponse>,
    /// Количество загрузок по сети (для статистики).
    network_fetches: u64,
    /// Количество загрузок из файлового кэша.
    cache_hits: u64,
    /// Количество ошибок загрузки.
    errors: u64,
}

impl TileManager {
    /// Создаёт менеджер тайлов с корневым каталогом кэша `cache_dir`.
    pub fn new(cache_dir: PathBuf, url_template: UrlTemplate) -> anyhow::Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()?;

        // reqwest::Client создаётся ВНУТРИ tokio runtime (block_on).
        let client = runtime.block_on(async {
            reqwest::Client::builder()
                .user_agent("karta/0.0.1 (https://github.com/example/karta)")
                .timeout(Duration::from_secs(30))
                .build()
        })?;

        let sas_cache = Arc::new(SasCache::new(cache_dir));
        let (sender, receiver) = std::sync::mpsc::channel();

        Ok(Self {
            runtime: Arc::new(runtime),
            client,
            sas_cache,
            url_template,
            pending_tiles: HashMap::new(),
            loaded_tiles: HashMap::new(),
            tile_order: Vec::new(),
            sender,
            receiver,
            network_fetches: 0,
            cache_hits: 0,
            errors: 0,
        })
    }

    /// Количество тайлов в памяти.
    pub fn loaded_count(&self) -> usize {
        self.loaded_tiles.len()
    }

    /// Возвращает список идентификаторов загруженных тайлов.
    pub fn loaded_tile_ids(&self) -> Vec<TileId> {
        self.loaded_tiles.keys().copied().collect()
    }

    /// Проверяет, загружен ли тайл в память.
    pub fn is_loaded(&self, tile: &TileId) -> bool {
        self.loaded_tiles.contains_key(tile)
    }

    /// Количество тайлов в процессе загрузки.
    pub fn pending_count(&self) -> usize {
        self.pending_tiles.len()
    }

    /// Возвращает загруженный тайл из памяти, если он есть.
    pub fn get_tile(&mut self, tile: &TileId) -> Option<&DynamicImage> {
        // Обновляем порядок доступа для LRU.
        if let Some(pos) = self.tile_order.iter().position(|t| t == tile) {
            self.tile_order.remove(pos);
            self.tile_order.push(*tile);
        }
        self.loaded_tiles.get(tile)
    }

    /// Проверяет наличие тайла в SAS.Planet кэше; при наличии загружает в память.
    fn try_load_from_cache(&mut self, tile: &TileId) -> bool {
        // SasCache::get_tile — async (trait TileCache), вызываем через block_on.
        // Это безопасно: мы не внутри async-контекста.
        let cache_data = self.runtime.block_on(self.sas_cache.get_tile(LAYER, tile));

        match cache_data {
            Ok(Some(data)) => match image::load_from_memory(&data) {
                Ok(img) => {
                    self.cache_hits += 1;
                    self.loaded_tiles.insert(*tile, img);
                    self.tile_order.push(*tile);
                    self.evict_old_tiles();
                    true
                }
                Err(e) => {
                    tracing::warn!(?tile, error = %e, "Не удалось декодировать тайл из кэша");
                    false
                }
            },
            Ok(None) => false,
            Err(e) => {
                tracing::warn!(?tile, error = %e, "Ошибка чтения тайла из кэша");
                false
            }
        }
    }

    /// Выгружает старые тайлы из памяти при превышении лимита.
    fn evict_old_tiles(&mut self) {
        while self.tile_order.len() > MAX_MEMORY_TILES {
            let oldest = self.tile_order.remove(0);
            self.loaded_tiles.remove(&oldest);
        }
    }

    /// Запрашивает загрузку тайла.
    ///
    /// 1. Проверяет `loaded_tiles` → если есть, возвращает.
    /// 2. Проверяет `pending_tiles` → если уже загружается, возвращает.
    /// 3. Проверяет SAS.Planet кэш → если есть, загружает в память.
    /// 4. Иначе — spawn async task для загрузки по HTTP.
    pub fn request_tile(&mut self, tile: &TileId) {
        // 1. Уже загружен.
        if self.loaded_tiles.contains_key(tile) {
            return;
        }

        // 2. Уже в процессе загрузки.
        if self.pending_tiles.contains_key(tile) {
            return;
        }

        // 3. Есть в файловом кэше SAS.Planet.
        if self.try_load_from_cache(tile) {
            return;
        }

        // 4. Загружаем по сети.
        tracing::debug!(?tile, "Загрузка тайла по сети");
        self.network_fetches += 1;
        self.pending_tiles.insert(*tile, Instant::now());

        let client = self.client.clone();
        let sas_cache = self.sas_cache.clone();
        let sender = self.sender.clone();
        let url = self.url_template.build_url(tile);
        let tile_id = *tile;

        self.runtime.spawn(async move {
            let response = fetch_and_cache_tile(&client, &sas_cache, tile_id, &url).await;
            let _ = sender.send(response);
        });
    }

    /// Обрабатывает ответы от фоновых задач загрузки.
    pub fn process_responses(&mut self) {
        while let Ok(response) = self.receiver.try_recv() {
            match response {
                TileResponse::Loaded(tile, img) => {
                    self.pending_tiles.remove(&tile);
                    self.loaded_tiles.insert(tile, img);
                    self.tile_order.push(tile);
                    self.evict_old_tiles();
                }
                TileResponse::Error(tile, error) => {
                    tracing::warn!(?tile, error = %error, "Ошибка загрузки тайла");
                    self.pending_tiles.remove(&tile);
                    self.errors += 1;
                }
            }
        }
    }
}

/// Загружает тайл по HTTP, кэширует его в SAS.Planet и декодирует.
async fn fetch_and_cache_tile(
    client: &reqwest::Client,
    sas_cache: &SasCache,
    tile: TileId,
    url: &str,
) -> TileResponse {
    let result = client.get(url).send().await;
    let bytes = match result {
        Ok(resp) => match resp.error_for_status() {
            Ok(resp) => match resp.bytes().await {
                Ok(bytes) => bytes,
                Err(e) => {
                    return TileResponse::Error(tile, format!("Не удалось прочитать тело: {}", e))
                }
            },
            Err(e) => return TileResponse::Error(tile, format!("HTTP статус ошибки: {}", e)),
        },
        Err(e) => return TileResponse::Error(tile, format!("Ошибка сети: {}", e)),
    };

    if bytes.is_empty() {
        return TileResponse::Error(tile, "Пустой ответ от сервера".to_string());
    }

    // Сохраняем в SAS.Planet кэш.
    if let Err(e) = sas_cache.put_tile(LAYER, &tile, &bytes).await {
        tracing::warn!(?tile, error = %e, "Не удалось записать тайл в кэш");
    }

    // Декодируем изображение.
    match image::load_from_memory(&bytes) {
        Ok(img) => TileResponse::Loaded(tile, img),
        Err(e) => TileResponse::Error(tile, format!("Не удалось декодировать PNG: {}", e)),
    }
}
