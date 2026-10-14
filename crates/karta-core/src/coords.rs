//! Географические координаты и преобразования.

use serde::{Deserialize, Serialize};

/// Географическая координата в системе WGS84.
///
/// `lat` — широта в диапазоне [-90.0, 90.0],
/// `lon` — долгота в диапазоне [-180.0, 180.0].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LatLon {
    /// Широта (degrees).
    pub lat: f64,
    /// Долгота (degrees).
    pub lon: f64,
}

impl LatLon {
    /// Создаёт новую координату.
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }

    /// Проверяет, что координата находится в допустимом диапазоне
    /// (lat ∈ [-90, 90], lon ∈ [-180, 180]).
    pub fn valid(&self) -> bool {
        (-90.0..=90.0).contains(&self.lat) && (-180.0..=180.0).contains(&self.lon)
    }
}

/// Экранная (пиксельная) координата относительно левого верхнего угла вьюпорта.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenCoord {
    /// Горизонтальная координата (pixels).
    pub x: f64,
    /// Вертикальная координата (pixels).
    pub y: f64,
}

/// Трейт для объектов, имеющих географические границы.
pub trait CoordBounds {
    /// Возвращает границы в формате `(юго-запад, северо-восток)`.
    fn bounds(&self) -> (LatLon, LatLon);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lat_lon_valid() {
        assert!(LatLon::new(0.0, 0.0).valid());
        assert!(LatLon::new(90.0, 180.0).valid());
        assert!(LatLon::new(-90.0, -180.0).valid());
    }

    #[test]
    fn test_lat_lon_invalid() {
        assert!(!LatLon::new(91.0, 0.0).valid());
        assert!(!LatLon::new(0.0, 181.0).valid());
        assert!(!LatLon::new(-91.0, 0.0).valid());
    }
}
