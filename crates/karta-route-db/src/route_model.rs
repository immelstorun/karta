//! Модель данных маршрута.

use chrono::{DateTime, Utc};
use karta_core::coords::LatLon;
use karta_core::vector::TrackPoint;
use serde::{Deserialize, Serialize};

/// Источник маршрута.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteSource {
    /// Импортирован из tlib.ru
    Tlib,
    /// Community-база маршрутов
    Community,
    /// Импортирован из GPX-файла
    Gpx,
    /// Создан вручную пользователем
    Manual,
}

/// Сложность маршрута.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    /// Лёгкий маршрут (прогулочный)
    Easy,
    /// Средней сложности
    Moderate,
    /// Сложный маршрут
    Hard,
    /// Экстремальный маршрут
    Extreme,
}

/// Маршрут — именованная последовательность точек с метаданными.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Уникальный идентификатор маршрута
    pub id: String,
    /// Название маршрута
    pub name: String,
    /// Описание маршрута
    pub description: Option<String>,
    /// Точки маршрута (нитка маршрута)
    pub points: Vec<TrackPoint>,
    /// Источник маршрута
    pub source: RouteSource,
    /// Сложность маршрута
    pub difficulty: Option<Difficulty>,
    /// Дата создания маршрута
    pub created_at: DateTime<Utc>,
    /// Автор маршрута
    pub author: Option<String>,
    /// Теги маршрута (например, "хайкинг", "водный")
    pub tags: Vec<String>,
}

impl Route {
    /// Вычисляет длину маршрута в метрах (сумма haversine-расстояний между точками).
    pub fn length_meters(&self) -> f64 {
        self.points
            .windows(2)
            .map(|w| karta_core::vector::haversine_distance(&w[0].lat_lon, &w[1].lat_lon))
            .sum()
    }

    /// Возвращает bounding box маршрута.
    ///
    /// Возвращает `(юго-запад, северо-восток)`.
    /// Если маршрут пуст — возвращает `None`.
    pub fn bounds(&self) -> Option<(LatLon, LatLon)> {
        if self.points.is_empty() {
            return None;
        }
        let mut min_lat = f64::MAX;
        let mut min_lon = f64::MAX;
        let mut max_lat = f64::MIN;
        let mut max_lon = f64::MIN;
        for point in &self.points {
            let lat = point.lat_lon.lat;
            let lon = point.lat_lon.lon;
            min_lat = min_lat.min(lat);
            min_lon = min_lon.min(lon);
            max_lat = max_lat.max(lat);
            max_lon = max_lon.max(lon);
        }
        Some((
            LatLon { lat: min_lat, lon: min_lon },
            LatLon { lat: max_lat, lon: max_lon },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moscow() -> TrackPoint {
        TrackPoint { lat_lon: LatLon::new(55.7558, 37.6173), elevation: None, time: None }
    }

    fn spb() -> TrackPoint {
        TrackPoint { lat_lon: LatLon::new(59.9343, 30.3351), elevation: None, time: None }
    }

    #[test]
    fn test_route_length() {
        let route = Route {
            id: "1".to_string(),
            name: "Test".to_string(),
            description: None,
            points: vec![moscow(), spb()],
            source: RouteSource::Manual,
            difficulty: None,
            created_at: Utc::now(),
            author: None,
            tags: vec![],
        };
        let length = route.length_meters();
        assert!((625_000.0..635_000.0).contains(&length), "length: {length}");
    }

    #[test]
    fn test_route_bounds() {
        let route = Route {
            id: "1".to_string(),
            name: "Test".to_string(),
            description: None,
            points: vec![moscow(), spb()],
            source: RouteSource::Manual,
            difficulty: None,
            created_at: Utc::now(),
            author: None,
            tags: vec![],
        };
        let bounds = route.bounds().expect("bounds should be Some");
        assert_eq!(bounds.0.lat, 55.7558);
        assert_eq!(bounds.1.lat, 59.9343);
    }

    #[test]
    fn test_empty_route_bounds_none() {
        let route = Route {
            id: "1".to_string(),
            name: "Test".to_string(),
            description: None,
            points: vec![],
            source: RouteSource::Manual,
            difficulty: None,
            created_at: Utc::now(),
            author: None,
            tags: vec![],
        };
        assert!(route.bounds().is_none());
    }
}
