//! Состояние карты: центр, zoom, панорамирование.

use karta_core::coords::LatLon;

/// Размер тайла в пикселях (стандарт Web Mercator).
pub const TILE_SIZE: f64 = 256.0;

/// Минимальный и максимальный zoom.
pub const MIN_ZOOM: f64 = 2.0;
pub const MAX_ZOOM: f64 = 19.0;

/// Начальная позиция карты (Эльбрус — сердце туристической России).
const INITIAL_CENTER: LatLon = LatLon {
    lat: 43.3499,
    lon: 42.4453,
};
const INITIAL_ZOOM: f64 = 10.0;

/// Состояние карты: географический центр и дробный zoom.
#[derive(Debug, Clone)]
pub struct MapState {
    /// Географический центр видимой области.
    pub center: LatLon,
    /// Дробный zoom (2.0..=19.0).
    pub zoom: f64,
}

impl Default for MapState {
    fn default() -> Self {
        Self::new()
    }
}

impl MapState {
    /// Создаёт состояние карты с начальными значениями.
    pub fn new() -> Self {
        Self {
            center: INITIAL_CENTER,
            zoom: INITIAL_ZOOM,
        }
    }

    /// Размер мира в пикселях при текущем zoom.
    pub fn world_size(&self) -> f64 {
        world_size(self.zoom)
    }

    /// Мировая пиксельная координата (XYZ) географической точки.
    pub fn lat_lon_to_world(&self, lat_lon: &LatLon) -> (f64, f64) {
        let ws = self.world_size();
        let lat = lat_lon.lat.clamp(-85.05112878, 85.05112878);
        let lon = lat_lon.lon.clamp(-180.0, 180.0);
        let x = (lon + 180.0) / 360.0 * ws;
        let lat_rad = lat.to_radians();
        let y = ((1.0 - lat_rad.tan().asinh() / std::f64::consts::PI) / 2.0) * ws;
        (x, y)
    }

    /// Обратное преобразование: мировая пиксельная координата → широта/долгота.
    pub fn world_to_lat_lon(&self, x: f64, y: f64) -> LatLon {
        let ws = self.world_size();
        let lon = x / ws * 360.0 - 180.0;
        let n = std::f64::consts::PI * (1.0 - 2.0 * y / ws);
        let lat = n.sinh().atan().to_degrees();
        LatLon::new(lat, lon)
    }

    /// Преобразует точку экрана (вьюпорта) в географическую координату.
    pub fn screen_to_lat_lon(&self, screen: (f64, f64), viewport_center: (f64, f64)) -> LatLon {
        let (cx, cy) = self.lat_lon_to_world(&self.center);
        let dx = screen.0 - viewport_center.0;
        let dy = screen.1 - viewport_center.1;
        self.world_to_lat_lon(cx + dx, cy + dy)
    }

    /// Преобразует географическую координату в точку экрана.
    pub fn lat_lon_to_screen(&self, lat_lon: &LatLon, viewport_center: (f64, f64)) -> (f64, f64) {
        let (cx, cy) = self.lat_lon_to_world(&self.center);
        let (px, py) = self.lat_lon_to_world(lat_lon);
        (viewport_center.0 + px - cx, viewport_center.1 + py - cy)
    }

    /// Панорамирует карту на `dx`, `dy` пикселей экрана.
    pub fn pan(&mut self, dx: f64, dy: f64) {
        let (cx, cy) = self.lat_lon_to_world(&self.center);
        let new_center = self.world_to_lat_lon(cx - dx, cy - dy);
        self.center = new_center;
    }

    /// Меняет zoom, сохраняя точку под курсором неподвижной (anchor zoom).
    pub fn zoom_at(&mut self, anchor: (f64, f64), viewport_center: (f64, f64), delta_zoom: f64) {
        let old_zoom = self.zoom;
        self.zoom = (self.zoom + delta_zoom).clamp(MIN_ZOOM, MAX_ZOOM);
        if (self.zoom - old_zoom).abs() < f64::EPSILON {
            return; // zoom упёрлся в предел — центр не меняем
        }

        // Географическая точка под курсором при старом zoom.
        let anchor_lat_lon = self.screen_to_lat_lon(anchor, viewport_center);

        // Новый центр: anchor должен остаться в той же точке экрана.
        let (ax, ay) = self.lat_lon_to_world(&anchor_lat_lon);
        let new_cx = ax - (anchor.0 - viewport_center.0);
        let new_cy = ay - (anchor.1 - viewport_center.1);
        self.center = self.world_to_lat_lon(new_cx, new_cy);
    }

    /// Приближает карту на один уровень.
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom + 1.0).min(MAX_ZOOM);
    }

    /// Отдаляет карту на один уровень.
    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom - 1.0).max(MIN_ZOOM);
    }

    /// Сбрасывает карту в начальное состояние.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

/// Размер мира в пикселях для заданного zoom.
pub fn world_size(zoom: f64) -> f64 {
    TILE_SIZE * 2f64.powf(zoom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_lat_lon_roundtrip() {
        let map = MapState::new();
        let viewport_center = (400.0, 300.0);
        let screen = (350.0, 280.0);
        let lat_lon = map.screen_to_lat_lon(screen, viewport_center);
        let back = map.lat_lon_to_screen(&lat_lon, viewport_center);
        assert!((back.0 - screen.0).abs() < 1e-6);
        assert!((back.1 - screen.1).abs() < 1e-6);
    }

    #[test]
    fn test_pan() {
        let mut map = MapState::new();
        let center_before = map.center;
        map.pan(100.0, 50.0);
        // После панорамирования прежний центр должен сместиться на экране
        // ровно на величину pan (карта «следует» за курсором).
        let screen = map.lat_lon_to_screen(&center_before, (0.0, 0.0));
        assert!((screen.0 - 100.0).abs() < 1e-6);
        assert!((screen.1 - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_zoom_clamp() {
        let mut map = MapState::new();
        for _ in 0..30 {
            map.zoom_in();
        }
        assert_eq!(map.zoom, MAX_ZOOM);
        for _ in 0..30 {
            map.zoom_out();
        }
        assert_eq!(map.zoom, MIN_ZOOM);
    }
}
