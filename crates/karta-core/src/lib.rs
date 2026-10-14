//! # karta-core
//!
//! Ядро картографической платформы karta. Содержит:
//! - Географические координаты и преобразования
//! - Тайловую математику (XYZ/TMS/QuadKey)
//! - Проекции (Web Mercator, WGS84)
//! - Векторную модель данных (треки, метки, маршруты)

pub mod coords;
pub mod projection;
pub mod tiles;
pub mod vector;
