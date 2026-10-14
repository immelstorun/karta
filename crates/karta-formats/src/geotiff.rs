//! GeoTIFF — геопривязанные растровые карты.
//!
//! TODO: полная реализация чтения GeoTIFF.
//!
//! GeoTIFF — это TIFF-файл с дополнительными GeoKeys в тегах:
//! - `ModelTiepointTag` (33922) — привязка изображения к координатам
//! - `ModelPixelScaleTag` (33550) — размер пикселя в единицах проекции
//! - `GeoKeyDirectoryTag` (34735) — ключи проекции (EPSG-коды и параметры)
//! - `GeoAsciiParamsTag` (34737) — строковые параметры (например, название проекции)

/// Ошибка обработки GeoTIFF.
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    /// Ошибка чтения файла.
    #[error("File read error: {0}")]
    FileRead(String),

    /// Некорректная структура GeoTIFF.
    #[error("Invalid GeoTIFF: {0}")]
    InvalidGeotiff(String),
}

/// Метаданные геопривязки GeoTIFF.
#[derive(Debug, Clone, Default)]
pub struct GeoTiffInfo {
    /// Ширина изображения (pixels).
    pub width: u32,
    /// Высота изображения (pixels).
    pub height: u32,
    /// EPSG-код проекции, если определён.
    pub epsg_code: Option<u16>,
}

/// Читает GeoTIFF-файл.
///
/// TODO: implement. Необходимо:
/// - парсинг структуры TIFF (IFD, теги)
/// - извлечение GeoKeys для проекций
/// - декодирование растровых данных (через image crate или tiff crate)
pub fn read_geotiff(path: &std::path::Path) -> Result<Vec<u8>, FormatError> {
    let _ = path;
    // TODO: implement GeoTIFF reading.
    Err(FormatError::InvalidGeotiff(
        "GeoTIFF reading is not yet implemented".to_string(),
    ))
}

/// Читает информацию о геопривязке GeoTIFF-файла.
///
/// TODO: implement.
pub fn read_geotiff_info(path: &std::path::Path) -> Result<GeoTiffInfo, FormatError> {
    let _ = path;
    // TODO: implement GeoTIFF info extraction.
    Err(FormatError::InvalidGeotiff(
        "GeoTIFF info reading is not yet implemented".to_string(),
    ))
}
