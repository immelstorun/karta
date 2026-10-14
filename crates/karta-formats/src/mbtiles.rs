//! MBTiles (SQLite-based, совместимо с MobAC) — чтение metadata.
//!
//! TODO: поддержка записи MBTiles (создание тайловых пакетов из слоя).
//! TODO: поддержка форматов pbf, jpg в metadata.format.

use rusqlite::Connection;
use std::path::Path;

/// Ошибка обработки MBTiles.
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    /// Ошибка SQLite.
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Некорректная структура MBTiles.
    #[error("Invalid MBTiles: {0}")]
    InvalidMbtiles(String),
}

/// Метаданные MBTiles-файла.
#[derive(Debug, Clone, Default)]
pub struct MbtilesMetadata {
    /// Название слоя.
    pub name: Option<String>,
    /// Формат тайлов (png, jpg, pbf).
    pub format: Option<String>,
    /// Границы слоя в формате "minLon,minLat,maxLon,maxLat".
    pub bounds: Option<String>,
}

/// Читает metadata-таблицу MBTiles-файла через rusqlite.
pub fn read_mbtiles_metadata(path: &Path) -> Result<MbtilesMetadata, FormatError> {
    let conn = Connection::open(path)?;
    let mut stmt = conn.prepare("SELECT name, value FROM metadata")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0).unwrap_or_default(),
            row.get::<_, String>(1).unwrap_or_default(),
        ))
    })?;

    let mut metadata = MbtilesMetadata::default();
    for row in rows {
        let (key, value) = row?;
        match key.as_str() {
            "name" => metadata.name = Some(value),
            "format" => metadata.format = Some(value),
            "bounds" => metadata.bounds = Some(value),
            _ => {}
        }
    }
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_mbtiles_metadata() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let db_path = dir.path().join("test.mbtiles");

        let conn = Connection::open(&db_path).expect("failed to open db");
        conn.execute_batch(
            "CREATE TABLE metadata (name TEXT PRIMARY KEY, value TEXT);
             CREATE TABLE tiles (zoom_level INTEGER, tile_column INTEGER, tile_row INTEGER, tile_data BLOB);
             INSERT INTO metadata (name, value) VALUES ('name', 'Test Layer');
             INSERT INTO metadata (name, value) VALUES ('format', 'png');
             INSERT INTO metadata (name, value) VALUES ('bounds', '37.0,55.0,38.0,56.0');",
        )
        .expect("failed to seed db");

        let metadata = read_mbtiles_metadata(&db_path).expect("read failed");
        assert_eq!(metadata.name.as_deref(), Some("Test Layer"));
        assert_eq!(metadata.format.as_deref(), Some("png"));
        assert_eq!(metadata.bounds.as_deref(), Some("37.0,55.0,38.0,56.0"));
    }
}
