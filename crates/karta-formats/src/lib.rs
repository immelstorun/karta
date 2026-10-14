//! # karta-formats
//!
//! Импорт/экспорт форматов для karta:
//! - GPX (GPS Exchange Format) — треки, waypoints
//! - KML/KMZ (Google Earth) — метки, треки
//! - GeoJSON — векторные данные
//! - MBTiles — пакеты тайлов
//! - JNX (Garmin BirdsEye) — растровые карты для Garmin
//! - GeoTIFF — геопривязанные растровые карты

pub mod geojson;
pub mod gpx;
pub mod geotiff;
pub mod jnx;
pub mod kml;
pub mod mbtiles;
