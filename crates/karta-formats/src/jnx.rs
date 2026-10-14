//! JNX (Garmin BirdsEye) — бинарный формат растровых карт для Garmin.
//!
//! TODO: полная реализация чтения/записи JNX.
//!
//! Формат JNX:
//! - Заголовок фиксированной длины с версией, product_id и смещениями таблиц
//! - Таблица уровней зума (scale, tile count)
//! - Таблица тайлов (top/left/bottom/right в projected meters, offset, length)
//! - JPEG-данные тайлов

use karta_core::coords::LatLon;

/// Ошибка обработки JNX.
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    /// Ошибка чтения бинарных данных.
    #[error("Binary read error: {0}")]
    BinaryRead(String),

    /// Некорректная структура JNX.
    #[error("Invalid JNX: {0}")]
    InvalidJnx(String),
}

/// Упрощённая структура заголовка JNX-файла.
#[derive(Debug, Clone)]
pub struct JnxHeader {
    /// Версия формата JNX.
    pub version: u16,
    /// Идентификатор продукта Garmin.
    pub product_id: u32,
    /// Название карты.
    pub map_name: String,
}

/// Читает заголовок JNX-файла.
///
/// TODO: implement. Заголовок JNX начинается с magic bytes,
/// затем следует версия формата, product_id и другие поля.
pub fn read_jnx_header(data: &[u8]) -> Result<JnxHeader, FormatError> {
    let _ = data;
    // TODO: implement JNX header parsing.
    Err(FormatError::InvalidJnx(
        "JNX header parsing is not yet implemented".to_string(),
    ))
}

/// Читает таблицу уровней зума из JNX-файла.
///
/// TODO: implement.
pub fn read_jnx_levels(data: &[u8]) -> Result<Vec<()>, FormatError> {
    let _ = data;
    // TODO: implement JNX zoom levels table parsing.
    Err(FormatError::InvalidJnx(
        "JNX levels parsing is not yet implemented".to_string(),
    ))
}

/// Вспомогательная функция для чтения little-endian u32.
fn read_u32_le(data: &[u8], offset: usize) -> Result<u32, FormatError> {
    data.get(offset..offset + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| {
            FormatError::BinaryRead(format!("not enough bytes for u32 at offset {}", offset))
        })
}

/// Читает bound-координаты тайла JNX (projected meters → LatLon).
///
/// TODO: implement.
pub fn jnx_tile_bounds(data: &[u8], tile_offset: usize) -> Result<(LatLon, LatLon), FormatError> {
    let _ = (data, tile_offset, read_u32_le);
    // TODO: implement JNX tile bounds parsing.
    Err(FormatError::InvalidJnx(
        "JNX tile bounds parsing is not yet implemented".to_string(),
    ))
}
