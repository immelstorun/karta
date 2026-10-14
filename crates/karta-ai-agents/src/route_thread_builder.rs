//! Построение нитки маршрута из последовательности топонимов.
//!
//! Нитка маршрута (route thread) — упорядоченная последовательность
//! географических точек, образующих линию маршрута на карте.
//!
//! Текущая реализация — заглушка, возвращающая пустой список.
//!
//! TODO: алгоритм построения оптимальной нитки маршрута:
//! - Решение задачи коммивояжёра (TSP) для минимизации длины
//! - Учёт рельефа (избегание крутых склонов)
//! - Учёт речной сети и существующих троп
//! - SRTM-данные для анализа рельефа

use karta_core::coords::LatLon;
use karta_core::vector::haversine_distance;
use karta_route_db::route_model::Route;

/// Построитель нитки маршрута.
pub struct RouteThreadBuilder {
    /// Максимальное расстояние между соседними точками (в метрах)
    max_point_spacing_m: f64,
}

impl RouteThreadBuilder {
    /// Создаёт новый построитель нитки маршрута.
    pub fn new() -> Self {
        Self {
            max_point_spacing_m: 1000.0,
        }
    }

    /// Строит нитку маршрута из последовательности географических точек.
    ///
    /// TODO: реализация алгоритма (интерполяция + сглаживание).
    /// Пока возвращает пустой список.
    pub fn build_thread(&self, _points: &[LatLon]) -> Vec<LatLon> {
        // TODO: route thread building algorithm
        Vec::new()
    }

    /// Вычисляет общую длину polyline (в метрах).
    pub fn polyline_length_m(points: &[LatLon]) -> f64 {
        points
            .windows(2)
            .map(|w| haversine_distance(&w[0], &w[1]))
            .sum()
    }

    /// Строит маршрут из точек и названия.
    pub fn build_route(&self, name: &str, points: &[LatLon]) -> Route {
        Route {
            id: format!("route-{}", chrono::Utc::now().timestamp_millis()),
            name: name.to_string(),
            description: None,
            points: points
                .iter()
                .map(|p| karta_core::vector::TrackPoint {
                    lat_lon: *p,
                    elevation: None,
                    time: None,
                })
                .collect(),
            source: karta_route_db::route_model::RouteSource::Manual,
            difficulty: None,
            created_at: chrono::Utc::now(),
            author: None,
            tags: Vec::new(),
        }
    }

    /// Возвращает максимальное расстояние между точками.
    pub fn max_point_spacing_m(&self) -> f64 {
        self.max_point_spacing_m
    }
}

impl Default for RouteThreadBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_thread_stub() {
        let builder = RouteThreadBuilder::new();
        let points = vec![LatLon::new(55.7558, 37.6173), LatLon::new(55.8500, 37.7000)];
        let thread = builder.build_thread(&points);
        assert!(thread.is_empty());
    }

    #[test]
    fn test_polyline_length_zero() {
        let length = RouteThreadBuilder::polyline_length_m(&[]);
        assert_eq!(length, 0.0);
    }

    #[test]
    fn test_polyline_length_single_point() {
        let length = RouteThreadBuilder::polyline_length_m(&[LatLon::new(55.0, 37.0)]);
        assert_eq!(length, 0.0);
    }

    #[test]
    fn test_polyline_length_two_points() {
        let points = vec![LatLon::new(55.7558, 37.6173), LatLon::new(59.9343, 30.3351)];
        let length = RouteThreadBuilder::polyline_length_m(&points);
        assert!(length > 0.0);
        assert!((625_000.0..635_000.0).contains(&length), "length: {length}");
    }

    #[test]
    fn test_build_route() {
        let builder = RouteThreadBuilder::new();
        let points = vec![LatLon::new(55.7558, 37.6173), LatLon::new(55.8500, 37.7000)];
        let route = builder.build_route("Test Route", &points);
        assert_eq!(route.name, "Test Route");
        assert_eq!(route.points.len(), 2);
        assert!(!route.id.is_empty());
    }
}
