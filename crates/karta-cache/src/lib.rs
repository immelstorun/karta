//! # karta-cache
//!
//! Подсистема кэширования тайлов. Поддерживает:
//! - SAS.Planet native cache (файловая структура z/x/y)
//! - MBTiles (SQLite-based, совместимо с MobAC)
//! - In-memory LRU кэш для горячего доступа

pub mod mbtiles;
pub mod memory;
pub mod sas_cache;
pub mod traits;

pub use memory::MemoryCache;
pub use mbtiles::MbtilesCache;
pub use sas_cache::SasCache;
pub use traits::{CacheError, CacheType, TileCache};
