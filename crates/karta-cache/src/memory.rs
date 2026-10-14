//! In-memory LRU-кэш тайлов для горячего доступа.

use async_trait::async_trait;
use karta_core::tiles::TileId;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex;

use crate::traits::{CacheError, CacheType, TileCache};

/// LRU-кэш тайлов в оперативной памяти.
pub struct MemoryCache {
    cache: Mutex<LruCache<(String, TileId), Vec<u8>>>,
}

impl MemoryCache {
    /// Создаёт новый in-memory кэш с заданной максимальной ёмкостью (в тайлах).
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity.max(1)).expect("capacity must be positive"),
            )),
        }
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::with_capacity(1024)
    }
}

#[async_trait]
impl TileCache for MemoryCache {
    async fn get_tile(&self, layer: &str, tile: &TileId) -> Result<Option<Vec<u8>>, CacheError> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| CacheError::ReadFailed("cache mutex poisoned".to_string()))?;
        Ok(cache.get(&(layer.to_string(), *tile)).cloned())
    }

    async fn put_tile(
        &self,
        layer: &str,
        tile: &TileId,
        data: &[u8],
    ) -> Result<(), CacheError> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| CacheError::WriteFailed("cache mutex poisoned".to_string()))?;
        cache.put((layer.to_string(), *tile), data.to_vec());
        Ok(())
    }

    async fn delete_tile(&self, layer: &str, tile: &TileId) -> Result<(), CacheError> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| CacheError::ReadFailed("cache mutex poisoned".to_string()))?;
        cache.pop(&(layer.to_string(), *tile));
        Ok(())
    }

    fn cache_type(&self) -> CacheType {
        CacheType::Memory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_cache_put_get_delete() {
        let cache = MemoryCache::default();
        let tile = TileId { x: 1, y: 2, zoom: 3 };
        let data = b"tile data".to_vec();

        cache.put_tile("osm", &tile, &data).await.unwrap();
        let loaded = cache.get_tile("osm", &tile).await.unwrap();
        assert_eq!(loaded, Some(data));

        cache.delete_tile("osm", &tile).await.unwrap();
        let loaded = cache.get_tile("osm", &tile).await.unwrap();
        assert_eq!(loaded, None);
    }

    #[tokio::test]
    async fn test_memory_cache_lru_eviction() {
        let cache = MemoryCache::with_capacity(2);
        let tile1 = TileId { x: 1, y: 1, zoom: 1 };
        let tile2 = TileId { x: 2, y: 2, zoom: 1 };
        let tile3 = TileId { x: 3, y: 3, zoom: 1 };

        cache.put_tile("osm", &tile1, b"a").await.unwrap();
        cache.put_tile("osm", &tile2, b"b").await.unwrap();
        // Touch tile1 to make tile2 the least recently used.
        cache.get_tile("osm", &tile1).await.unwrap();
        cache.put_tile("osm", &tile3, b"c").await.unwrap();

        assert!(cache.get_tile("osm", &tile2).await.unwrap().is_none());
        assert!(cache.get_tile("osm", &tile1).await.unwrap().is_some());
        assert!(cache.get_tile("osm", &tile3).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_memory_cache_layers_are_independent() {
        let cache = MemoryCache::default();
        let tile = TileId { x: 1, y: 1, zoom: 1 };

        cache.put_tile("osm", &tile, b"osm").await.unwrap();
        assert!(cache.get_tile("sat", &tile).await.unwrap().is_none());
        assert_eq!(
            cache.get_tile("osm", &tile).await.unwrap(),
            Some(b"osm".to_vec())
        );
    }
}
