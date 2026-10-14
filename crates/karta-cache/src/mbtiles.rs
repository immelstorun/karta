//! MBTiles-кэш (SQLite-based, совместимо с MobAC).
//!
//! Схема БД: таблица `metadata` (name TEXT PRIMARY KEY, value TEXT)
//! + таблица `tiles` (zoom_level INTEGER, tile_column INTEGER, tile_row INTEGER, tile_data BLOB).
//!
//! TODO: поддержка `grids`/`grid_data` (vector tiles, UTFGrid).
//! TODO: поддержка форматов pbf, jpg в metadata.format.

use async_trait::async_trait;
use karta_core::tiles::TileId;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

use crate::traits::{CacheError, CacheType, TileCache};

/// MBTiles-кэш на базе SQLite.
pub struct MbtilesCache {
    conn: Mutex<Connection>,
}

impl MbtilesCache {
    /// Открывает существующую MBTiles-базу (или создаёт новую, если файла нет).
    pub fn open(path: &Path) -> Result<Self, CacheError> {
        let conn = Connection::open(path)
            .map_err(|e| CacheError::ReadFailed(format!("{}: {}", path.display(), e)))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Создаёт новую MBTiles-базу с таблицами `metadata` и `tiles`.
    pub fn create(path: &Path) -> Result<Self, CacheError> {
        let cache = Self::open(path)?;
        {
            let conn = cache.connection();
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS metadata (
                    name TEXT PRIMARY KEY,
                    value TEXT
                );
                CREATE TABLE IF NOT EXISTS tiles (
                    zoom_level INTEGER,
                    tile_column INTEGER,
                    tile_row INTEGER,
                    tile_data BLOB,
                    PRIMARY KEY (zoom_level, tile_column, tile_row)
                );",
            )?;
        }
        Ok(cache)
    }

    /// Возвращает прямую ссылку на SQLite-соединение (для расширенных операций).
    pub fn connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("mbtiles connection mutex poisoned")
    }
}

#[async_trait]
impl TileCache for MbtilesCache {
    async fn get_tile(&self, layer: &str, tile: &TileId) -> Result<Option<Vec<u8>>, CacheError> {
        // MBTiles uses TMS y-coordinate scheme (origin at bottom-left).
        let max_y = (1u64 << tile.zoom) - 1;
        let tms_row = max_y - tile.y as u64;

        let conn = self.connection();
        let mut stmt = conn
            .prepare("SELECT tile_data FROM tiles WHERE zoom_level = ?1 AND tile_column = ?2 AND tile_row = ?3")?;
        let mut rows = stmt.query([
            (tile.zoom as i64),
            (tile.x as i64),
            (tms_row as i64),
        ])?;
        if let Some(row) = rows.next()? {
            let data: Vec<u8> = row.get(0)?;
            let _ = layer; // MBTiles is a single-layer format; layer name is ignored.
            return Ok(Some(data));
        }
        Ok(None)
    }

    async fn put_tile(
        &self,
        layer: &str,
        tile: &TileId,
        data: &[u8],
    ) -> Result<(), CacheError> {
        let _ = layer; // MBTiles is a single-layer format; layer name is ignored.
        let max_y = (1u64 << tile.zoom) - 1;
        let tms_row = max_y - tile.y as u64;
        let conn = self.connection();
        conn.execute(
            "INSERT OR REPLACE INTO tiles (zoom_level, tile_column, tile_row, tile_data) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                tile.zoom as i64,
                tile.x as i64,
                tms_row as i64,
                data
            ],
        )?;
        Ok(())
    }

    async fn delete_tile(&self, layer: &str, tile: &TileId) -> Result<(), CacheError> {
        let _ = layer;
        let max_y = (1u64 << tile.zoom) - 1;
        let tms_row = max_y - tile.y as u64;
        let conn = self.connection();
        conn.execute(
            "DELETE FROM tiles WHERE zoom_level = ?1 AND tile_column = ?2 AND tile_row = ?3",
            rusqlite::params![
                tile.zoom as i64,
                tile.x as i64,
                tms_row as i64
            ],
        )?;
        Ok(())
    }

    fn cache_type(&self) -> CacheType {
        CacheType::Mbtiles
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karta_core::tiles::TileId;

    #[tokio::test]
    async fn test_mbtiles_create_put_get_delete() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let db_path = dir.path().join("test.mbtiles");
        let cache = MbtilesCache::create(&db_path).expect("failed to create mbtiles");

        let tile = TileId { x: 5, y: 10, zoom: 7 };
        let data = b"tile bytes".to_vec();

        cache.put_tile("layer", &tile, &data).await.expect("put failed");
        let loaded = cache.get_tile("layer", &tile).await.expect("get failed");
        assert_eq!(loaded, Some(data.clone()));

        cache.delete_tile("layer", &tile).await.expect("delete failed");
        let loaded = cache.get_tile("layer", &tile).await.expect("get failed");
        assert_eq!(loaded, None);
    }

    #[tokio::test]
    async fn test_mbtiles_tms_y_conversion() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let db_path = dir.path().join("test.mbtiles");
        let cache = MbtilesCache::create(&db_path).expect("failed to create mbtiles");

        let tile = TileId { x: 3, y: 3, zoom: 3 };
        let data = b"data".to_vec();
        cache.put_tile("layer", &tile, &data).await.unwrap();

        // On zoom 3 there are 8 tiles per axis; y=3 (XYZ) maps to tms_row=4.
        let conn = cache.connection();
        let mut stmt = conn
            .prepare("SELECT tile_row FROM tiles")
            .expect("prepare failed");
        let tms_row: i64 = stmt
            .query_row([], |row| row.get(0))
            .expect("query failed");
        assert_eq!(tms_row, 4);
    }
}
