//! SAS.Planet-совместимый файловый кэш тайлов.
//!
//! Формат хранения:
//! `cache_root/layer_name/z{zoom+1}/{x div 1024}/x{x}/{y div 1024}/y{y}.{ext}`
//!
//! Файл `y{y}.tne` (tile not exists) — маркер отсутствия тайла:
//! содержит timestamp последней проверки.

use async_trait::async_trait;
use karta_core::tiles::TileId;
use std::path::PathBuf;

use crate::traits::{CacheError, CacheType, TileCache};

/// Файловый кэш тайлов, совместимый с SAS.Planet.
pub struct SasCache {
    /// Корневой каталог кэша.
    root: PathBuf,
}

impl SasCache {
    /// Создаёт новый SAS.Planet-кэш с корневым каталогом `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Возвращает путь к файлу тайла.
    ///
    /// SAS.Planet хранит:
    /// - уровень зума как `z{zoom + 1}` (1-based);
    /// - x и y, разбитыми на подкаталоги по 1024 тайла;
    /// - расширение файла задаётся слоем (обычно `png` или `jpg`).
    pub fn tile_path(&self, layer: &str, tile: &TileId, ext: &str) -> PathBuf {
        self.root
            .join(layer)
            .join(format!("z{}", tile.zoom + 1))
            .join(format!("{}", tile.x / 1024))
            .join(format!("x{}", tile.x))
            .join(format!("{}", tile.y / 1024))
            .join(format!("y{}.{}", tile.y, ext))
    }

    /// Возвращает путь к `.tne`-маркеру (tile not exists) для тайла.
    pub fn tne_path(&self, layer: &str, tile: &TileId) -> PathBuf {
        self.tile_path(layer, tile, "tne").with_extension("tne")
    }

    /// Убирает `.tne`-маркер для тайла (если он был).
    fn remove_tne_marker(&self, layer: &str, tile: &TileId) -> Result<(), CacheError> {
        let tne_path = self.tne_path(layer, tile);
        if tne_path.exists() {
            std::fs::remove_file(&tne_path)
                .map_err(|e| CacheError::WriteFailed(format!("{}: {}", tne_path.display(), e)))?;
        }
        Ok(())
    }

    /// Записывает `.tne`-маркер (тайл не существует).
    pub fn put_tne_marker(&self, layer: &str, tile: &TileId) -> Result<(), CacheError> {
        let tne_path = self.tne_path(layer, tile);
        if let Some(parent) = tne_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                CacheError::DirectoryNotFound(format!("{}: {}", parent.display(), e))
            })?;
        }
        // Содержимое — timestamp последней проверки.
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        std::fs::write(&tne_path, timestamp.to_string())
            .map_err(|e| CacheError::WriteFailed(format!("{}: {}", tne_path.display(), e)))
    }

    /// Читает тайл с диска, обходя `.tne`-маркеры и расширения файлов.
    fn read_tile_from_disk(
        &self,
        layer: &str,
        tile: &TileId,
    ) -> Result<Option<Vec<u8>>, CacheError> {
        for ext in ["png", "jpg", "jpeg"] {
            let path = self.tile_path(layer, tile, ext);
            if let Ok(data) = std::fs::read(&path) {
                return Ok(Some(data));
            }
        }
        Ok(None)
    }

    /// Записывает тайл на диск, создавая необходимые подкаталоги.
    fn write_tile_to_disk(
        &self,
        layer: &str,
        tile: &TileId,
        ext: &str,
        data: &[u8],
    ) -> Result<(), CacheError> {
        let path = self.tile_path(layer, tile, ext);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                CacheError::DirectoryNotFound(format!("{}: {}", parent.display(), e))
            })?;
        }
        std::fs::write(&path, data)
            .map_err(|e| CacheError::WriteFailed(format!("{}: {}", path.display(), e)))
    }
}

#[async_trait]
impl TileCache for SasCache {
    async fn get_tile(&self, layer: &str, tile: &TileId) -> Result<Option<Vec<u8>>, CacheError> {
        // Check .tne marker first — its presence means the tile does not exist.
        if self.tne_path(layer, tile).exists() {
            return Ok(None);
        }
        self.read_tile_from_disk(layer, tile)
    }

    async fn put_tile(&self, layer: &str, tile: &TileId, data: &[u8]) -> Result<(), CacheError> {
        // Default extension for SAS.Planet cache tiles is png.
        self.write_tile_to_disk(layer, tile, "png", data)?;
        // Writing the tile invalidates any "not exists" marker.
        self.remove_tne_marker(layer, tile)
    }

    async fn delete_tile(&self, layer: &str, tile: &TileId) -> Result<(), CacheError> {
        let mut last_err: Option<std::io::Error> = None;
        for ext in ["png", "jpg", "jpeg"] {
            let path = self.tile_path(layer, tile, ext);
            match std::fs::remove_file(&path) {
                Ok(()) => return Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => {
                    last_err = Some(e);
                    break;
                }
            }
        }
        // Also remove .tne marker if present.
        let tne_path = self.tne_path(layer, tile);
        if tne_path.exists() {
            if let Err(e) = std::fs::remove_file(&tne_path) {
                last_err = Some(e);
            }
        }
        match last_err {
            Some(e) => Err(CacheError::ReadFailed(format!(
                "failed to delete tile: {}",
                e
            ))),
            None => Ok(()),
        }
    }

    fn cache_type(&self) -> CacheType {
        CacheType::SasPlanet
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karta_core::tiles::TileId;

    fn temp_cache() -> (SasCache, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let cache = SasCache::new(dir.path());
        (cache, dir)
    }

    #[tokio::test]
    async fn test_sas_cache_put_get_delete() {
        let (cache, _dir) = temp_cache();
        let tile = TileId {
            x: 1234,
            y: 5678,
            zoom: 10,
        };
        let data = b"fake png data".to_vec();

        cache
            .put_tile("osm", &tile, &data)
            .await
            .expect("put failed");
        let loaded = cache.get_tile("osm", &tile).await.expect("get failed");
        assert_eq!(loaded, Some(data));

        cache
            .delete_tile("osm", &tile)
            .await
            .expect("delete failed");
        let loaded = cache.get_tile("osm", &tile).await.expect("get failed");
        assert_eq!(loaded, None);
    }

    #[tokio::test]
    async fn test_sas_cache_tne_marker() {
        let (cache, _dir) = temp_cache();
        let tile = TileId {
            x: 1,
            y: 2,
            zoom: 3,
        };

        // Simulate writing a .tne marker (tile not exists).
        let tne_path = cache.tne_path("osm", &tile);
        std::fs::create_dir_all(tne_path.parent().unwrap()).unwrap();
        std::fs::write(&tne_path, "1700000000").unwrap();

        // Tile with a .tne marker must be reported as missing.
        let loaded = cache.get_tile("osm", &tile).await.expect("get failed");
        assert_eq!(loaded, None);
    }

    #[test]
    fn test_tile_path_format() {
        let dir = tempfile::tempdir().unwrap();
        let cache = SasCache::new(dir.path());
        let tile = TileId {
            x: 1234,
            y: 567,
            zoom: 10,
        };
        let path = cache.tile_path("osm", &tile, "png");
        let expected = dir
            .path()
            .join("osm")
            .join("z11")
            .join("1")
            .join("x1234")
            .join("0")
            .join("y567.png");
        assert_eq!(path, expected);
    }
}
