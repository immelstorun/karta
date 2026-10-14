//! Картографические проекции: Web Mercator (EPSG:3857), WGS84 (EPSG:4326).

use crate::coords::LatLon;

/// Радиус Земли в метрах (сферическая модель, WGS84 Web Mercator).
const EARTH_RADIUS_M: f64 = 6_378_137.0;

/// Общий интерфейс картографической проекции.
pub trait Projection {
    /// Проецирует географическую координату в плоские координаты (метры).
    fn project(&self, coord: &LatLon) -> (f64, f64);

    /// Выполняет обратное преобразование из плоских координат.
    fn unproject(&self, x: f64, y: f64) -> LatLon;

    /// Возвращает разрешение (meters/pixel) на данном уровне зума.
    fn resolution(&self, zoom: u8, tile_size: u32) -> f64;
}

/// Проекция Web Mercator (EPSG:3857 / Pseudo-Mercator).
///
/// Используется подавляющим большинством веб-карт (OSM, Google, Bing, Yandex).
pub struct WebMercator;

impl WebMercator {
    /// Преобразует координату WGS84 в метры Web Mercator.
    ///
    /// Возвращает `(x, y)` в метрах; x ∈ [-20037508.34, 20037508.34].
    pub fn lat_lon_to_meters(lat: f64, lon: f64) -> (f64, f64) {
        let x = lon.to_radians() * EARTH_RADIUS_M;
        let y = lat.to_radians().tan().asinh() * EARTH_RADIUS_M;
        (x, y)
    }

    /// Преобразует метры Web Mercator обратно в координату WGS84.
    ///
    /// Возвращает `(lat, lon)` в градусах.
    pub fn meters_to_lat_lon(x: f64, y: f64) -> (f64, f64) {
        let lon = (x / EARTH_RADIUS_M).to_degrees();
        let lat = (y / EARTH_RADIUS_M).sinh().atan().to_degrees();
        (lat, lon)
    }

    /// Возвращает разрешение (meters/pixel) на данном уровне зума.
    pub fn resolution(zoom: u8, tile_size: u32) -> f64 {
        let total_width = 2.0 * std::f64::consts::PI * EARTH_RADIUS_M;
        total_width / (tile_size as f64 * f64::from(1u32 << zoom))
    }
}

impl Projection for WebMercator {
    fn project(&self, coord: &LatLon) -> (f64, f64) {
        Self::lat_lon_to_meters(coord.lat, coord.lon)
    }

    fn unproject(&self, x: f64, y: f64) -> LatLon {
        let (lat, lon) = Self::meters_to_lat_lon(x, y);
        LatLon::new(lat, lon)
    }

    fn resolution(&self, zoom: u8, tile_size: u32) -> f64 {
        Self::resolution(zoom, tile_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meters_roundtrip() {
        let (x, y) = WebMercator::lat_lon_to_meters(55.7558, 37.6173);
        let (lat, lon) = WebMercator::meters_to_lat_lon(x, y);
        assert!((lat - 55.7558).abs() < 1e-9);
        assert!((lon - 37.6173).abs() < 1e-9);
    }

    #[test]
    fn test_origin() {
        let (x, y) = WebMercator::lat_lon_to_meters(0.0, 0.0);
        assert!(x.abs() < 1e-9);
        assert!(y.abs() < 1e-9);
    }

    #[test]
    fn test_resolution() {
        // On zoom 0 the whole world (40075016.686 m) fits into one 256px tile.
        let res = WebMercator::resolution(0, 256);
        assert!((res - 40_075_016.686 / 256.0).abs() < 1.0);

        // Each zoom level halves the resolution.
        let res0 = WebMercator::resolution(0, 256);
        let res1 = WebMercator::resolution(1, 256);
        assert!((res0 / res1 - 2.0).abs() < 1e-9);
    }
}
