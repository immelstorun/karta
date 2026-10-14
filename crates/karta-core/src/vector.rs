//! Векторная модель данных: треки, метки, маршруты.

use crate::coords::LatLon;
use serde::{Deserialize, Serialize};

/// Точка трека с опциональной высотой и временем.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackPoint {
    /// Географическая координата точки.
    pub lat_lon: LatLon,
    /// Высота над уровнем моря (meters), если известна.
    pub elevation: Option<f64>,
    /// Момент времени фиксации точки (UTC), если известен.
    pub time: Option<chrono::DateTime<chrono::Utc>>,
}

/// GPX-трек — именованная последовательность точек.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    /// Название трека.
    pub name: String,
    /// Упорядоченный список точек трека.
    pub points: Vec<TrackPoint>,
}

/// Метка (waypoint) — именованная точка на карте.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Waypoint {
    /// Название метки.
    pub name: String,
    /// Географическая координата метки.
    pub lat_lon: LatLon,
    /// Произвольное текстовое описание.
    pub description: Option<String>,
}

/// Маршрут — именованная последовательность точек (без временных атрибутов).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Route {
    /// Название маршрута.
    pub name: String,
    /// Упорядоченный список точек маршрута.
    pub points: Vec<LatLon>,
}

impl Track {
    /// Вычисляет суммарную длину трека в метрах по формуле haversine.
    pub fn length_meters(&self) -> f64 {
        self.points
            .windows(2)
            .map(|w| haversine_distance(&w[0].lat_lon, &w[1].lat_lon))
            .sum()
    }
}

/// Вычисляет большое круговое расстояние между двумя точками (meters)
/// по формуле haversine (сферическая модель Земли, R = 6371000 m).
pub fn haversine_distance(a: &LatLon, b: &LatLon) -> f64 {
    const EARTH_RADIUS_M: f64 = 6_371_000.0;

    let lat1 = a.lat.to_radians();
    let lat2 = b.lat.to_radians();
    let dlat = (b.lat - a.lat).to_radians();
    let dlon = (b.lon - a.lon).to_radians();

    let h = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_M * h.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moscow() -> LatLon {
        LatLon::new(55.7558, 37.6173)
    }

    fn spb() -> LatLon {
        LatLon::new(59.9343, 30.3351)
    }

    #[test]
    fn test_haversine_zero() {
        let d = haversine_distance(&moscow(), &moscow());
        assert!(d.abs() < 1e-6);
    }

    #[test]
    fn test_haversine_moscow_spb() {
        // Moscow–Saint Petersburg is roughly 630 km.
        let d = haversine_distance(&moscow(), &spb());
        assert!((625_000.0..635_000.0).contains(&d), "distance: {d}");
    }

    #[test]
    fn test_track_length() {
        let track = Track {
            name: "test".to_string(),
            points: vec![
                TrackPoint {
                    lat_lon: moscow(),
                    elevation: None,
                    time: None,
                },
                TrackPoint {
                    lat_lon: spb(),
                    elevation: None,
                    time: None,
                },
            ],
        };
        let len = track.length_meters();
        assert!((625_000.0..635_000.0).contains(&len), "length: {len}");
    }
}
