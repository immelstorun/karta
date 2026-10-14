//! Тайловая математика: схемы XYZ/TMS, QuadKey.

use serde::{Deserialize, Serialize};

/// Идентификатор тайла в схеме XYZ (origin — левый верхний угол).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileId {
    /// Колонка тайла.
    pub x: u32,
    /// Строка тайла.
    pub y: u32,
    /// Уровень масштабирования.
    pub zoom: u8,
}

/// Преобразует географическую координату в идентификатор тайла (схема XYZ).
pub fn lat_lon_to_tile(lat: f64, lon: f64, zoom: u8) -> TileId {
    let n = f64::from(1u32 << zoom);
    let clamped_lat = lat.clamp(-85.05112878, 85.05112878);
    let x = ((lon + 180.0) / 360.0 * n).floor();
    let lat_rad = clamped_lat.to_radians();
    let y = ((1.0 - lat_rad.tan().asinh() / std::f64::consts::PI) / 2.0 * n).floor();
    TileId {
        x: x.max(0.0) as u32,
        y: y.max(0.0) as u32,
        zoom,
    }
}

/// Возвращает координаты левого верхнего угла тайла (широта, долгота).
pub fn tile_to_lat_lon(tile: &TileId) -> (f64, f64) {
    let n = f64::from(1u32 << tile.zoom);
    let lon = tile.x as f64 / n * 360.0 - 180.0;
    let lat_rad = (std::f64::consts::PI * (1.0 - 2.0 * tile.y as f64 / n))
        .sinh()
        .atan();
    let lat = lat_rad.to_degrees();
    (lat, lon)
}

/// Возвращает количество тайлов вдоль одной оси на данном уровне зума (2^zoom).
pub fn tile_count(zoom: u8) -> u64 {
    1u64 << zoom
}

/// Вычисляет Bing/Google QuadKey для тайла.
///
/// QuadKey — строка из цифр 0-3, где каждая цифра кодирует квадрант
/// на соответствующем уровне зума.
pub fn quadkey(tile: &TileId) -> String {
    let mut key = String::with_capacity(tile.zoom as usize);
    for z in (1..=tile.zoom).rev() {
        let mut digit = 0u8;
        let mask = 1u32 << (z - 1);
        if tile.x & mask != 0 {
            digit += 1;
        }
        if tile.y & mask != 0 {
            digit += 2;
        }
        key.push((b'0' + digit) as char);
    }
    key
}

/// Пиксельные координаты тайла.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TilePixel {
    /// X-координата (pixels).
    pub x: u32,
    /// Y-координата (pixels).
    pub y: u32,
    /// Размер тайла в пикселях (обычно 256).
    pub size: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lat_lon_to_tile() {
        let tile = lat_lon_to_tile(0.0, 0.0, 1);
        assert_eq!((tile.x, tile.y), (1, 1));

        let tile = lat_lon_to_tile(85.05, -180.0, 2);
        assert_eq!((tile.x, tile.y), (0, 0));

        let tile = lat_lon_to_tile(-85.05, 179.9, 2);
        assert_eq!((tile.x, tile.y), (3, 3));
    }

    #[test]
    fn test_tile_roundtrip() {
        let original = TileId { x: 3, y: 5, zoom: 7 };
        let (lat, lon) = tile_to_lat_lon(&original);
        let back = lat_lon_to_tile(lat, lon, original.zoom);
        assert_eq!(original, back);
    }

    #[test]
    fn test_tile_count() {
        assert_eq!(tile_count(0), 1);
        assert_eq!(tile_count(1), 2);
        assert_eq!(tile_count(8), 256);
    }

    #[test]
    fn test_quadkey() {
        let tile = TileId { x: 3, y: 5, zoom: 3 };
        assert_eq!(quadkey(&tile), "213");
    }
}
