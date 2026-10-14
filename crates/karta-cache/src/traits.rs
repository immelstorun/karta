//! Общий интерфейс кэшей тайлов.

use karta_core::tiles::TileId;

/// Тип кэша тайлов.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheType {
    /// Файловый кэш SAS.Planet.
    SasPlanet,
    /// SQLite-кэш MBTiles.
    Mbtiles,
    /// Кэш в оперативной памяти.
    Memory,
    /// Пользовательский тип кэша.
    Custom(String),
}

/// Ошибки кэширования тайлов.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    /// Каталог кэша не найден.
    #[error("Cache directory not found: {0}")]
    DirectoryNotFound(String),

    /// Ошибка чтения тайла.
    #[error("Failed to read tile: {0}")]
    ReadFailed(String),

    /// Ошибка записи тайла.
    #[error("Failed to write tile: {0}")]
    WriteFailed(String),

    /// Ошибка SQLite.
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// Асинхронный интерфейс кэша тайлов.
#[async_trait::async_trait]
pub trait TileCache: Send + Sync {
    /// Читает тайл из кэша. Возвращает `None`, если тайл отсутствует
    /// (в том числе при наличии `.tne`-маркера для SAS.Planet-кэша).
    async fn get_tile(&self, layer: &str, tile: &TileId) -> Result<Option<Vec<u8>>, CacheError>;

    /// Записывает тайл в кэш.
    async fn put_tile(&self, layer: &str, tile: &TileId, data: &[u8]) -> Result<(), CacheError>;

    /// Удаляет тайл из кэша.
    async fn delete_tile(&self, layer: &str, tile: &TileId) -> Result<(), CacheError>;

    /// Возвращает тип кэша.
    fn cache_type(&self) -> CacheType;
}
