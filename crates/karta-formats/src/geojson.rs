//! GeoJSON (RFC 7946) — импорт и экспорт векторных данных.

use karta_core::vector::Track;
use serde::{Deserialize, Serialize};

/// Ошибка обработки GeoJSON.
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    /// Ошибка JSON-парсинга.
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    /// Некорректная структура GeoJSON.
    #[error("Invalid GeoJSON: {0}")]
    InvalidGeojson(String),
}

/// Внутренняя модель FeatureCollection для десериализации.
#[derive(Debug, Deserialize)]
struct FeatureCollection {
    #[serde(default)]
    features: Vec<Feature>,
}

/// Внутренняя модель Feature.
#[derive(Debug, Deserialize)]
struct Feature {
    #[serde(default)]
    properties: serde_json::Value,
    geometry: Geometry,
}

/// Внутренняя модель Geometry.
#[derive(Debug, Deserialize)]
struct Geometry {
    #[serde(rename = "type")]
    geometry_type: String,
    coordinates: serde_json::Value,
}

/// Внутренняя модель FeatureCollection для сериализации.
#[derive(Debug, Serialize)]
struct ExportFeatureCollection {
    features: Vec<ExportFeature>,
}

/// Внутренняя модель Feature для сериализации.
#[derive(Debug, Serialize)]
struct ExportFeature {
    #[serde(rename = "type")]
    feature_type: &'static str,
    properties: ExportProperties,
    geometry: ExportGeometry,
}

/// Свойства экспортируемого feature.
#[derive(Debug, Serialize)]
struct ExportProperties {
    name: String,
}

/// Геометрия экспортируемого feature.
#[derive(Debug, Serialize)]
struct ExportGeometry {
    #[serde(rename = "type")]
    geometry_type: &'static str,
    coordinates: Vec<Vec<f64>>,
}

/// Парсит GeoJSON FeatureCollection с LineString features.
pub fn parse_geojson(data: &str) -> Result<Vec<Track>, FormatError> {
    let fc: FeatureCollection = serde_json::from_str(data)?;

    let mut tracks = Vec::new();
    for feature in fc.features {
        if feature.geometry.geometry_type != "LineString" {
            continue;
        }

        let coordinates: Vec<Vec<f64>> = serde_json::from_value(
            feature.geometry.coordinates,
        )
        .map_err(|e| FormatError::InvalidGeojson(format!("invalid coordinates: {}", e)))?;

        let name = feature
            .properties
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed")
            .to_string();

        let points: Vec<karta_core::vector::TrackPoint> = coordinates
            .iter()
            .map(|c| {
                let lat = *c.get(1).ok_or_else(|| {
                    FormatError::InvalidGeojson("missing latitude".to_string())
                })?;
                let lon = *c.get(0).ok_or_else(|| {
                    FormatError::InvalidGeojson("missing longitude".to_string())
                })?;
                Ok(karta_core::vector::TrackPoint {
                    lat_lon: karta_core::coords::LatLon::new(lat, lon),
                    elevation: c.get(2).copied(),
                    time: None,
                })
            })
            .collect::<Result<Vec<_>, FormatError>>()?;

        tracks.push(Track { name, points });
    }

    Ok(tracks)
}

/// Экспортирует треки в GeoJSON FeatureCollection.
pub fn export_geojson(tracks: &[Track]) -> Result<String, FormatError> {
    let features = tracks
        .iter()
        .map(|track| ExportFeature {
            feature_type: "Feature",
            properties: ExportProperties {
                name: track.name.clone(),
            },
            geometry: ExportGeometry {
                geometry_type: "LineString",
                coordinates: track
                    .points
                    .iter()
                    .map(|p| {
                        let mut coord = vec![p.lat_lon.lon, p.lat_lon.lat];
                        if let Some(elevation) = p.elevation {
                            coord.push(elevation);
                        }
                        coord
                    })
                    .collect(),
            },
        })
        .collect();

    let fc = ExportFeatureCollection { features };
    Ok(serde_json::to_string(&fc)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_GEOJSON: &str = r#"{
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": { "name": "Test Track" },
                "geometry": {
                    "type": "LineString",
                    "coordinates": [
                        [37.6173, 55.7558],
                        [37.6175, 55.7560]
                    ]
                }
            }
        ]
    }"#;

    #[test]
    fn test_parse_geojson() {
        let tracks = parse_geojson(SAMPLE_GEOJSON).expect("parse failed");
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].name, "Test Track");
        assert_eq!(tracks[0].points.len(), 2);
        assert!((tracks[0].points[0].lat_lon.lat - 55.7558).abs() < 1e-6);
        assert!((tracks[0].points[0].lat_lon.lon - 37.6173).abs() < 1e-6);
    }

    #[test]
    fn test_geojson_roundtrip() {
        let tracks = parse_geojson(SAMPLE_GEOJSON).unwrap();
        let exported = export_geojson(&tracks).expect("export failed");
        let reparsed = parse_geojson(&exported).expect("reparse failed");
        assert_eq!(reparsed.len(), 1);
        assert_eq!(reparsed[0].name, "Test Track");
        assert_eq!(reparsed[0].points.len(), 2);
    }
}
