//! KML (Keyhole Markup Language) — метки, треки.
//!
//! TODO: реализовать полный парсинг KML:
//! - `<Placemark>` с `<Point>` → waypoints
//! - `<Placemark>` с `<LineString>`/`<Track>` → треки
//! - Namespace-обработка (`http://www.opengis.net/kml/2.2`)
//! - Поддержка `<MultiGeometry>`, `<Style>`, `<ExtendedData>`

use karta_core::vector::Track;

/// Ошибка разбора KML.
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    /// Ошибка XML-парсинга.
    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::Error),

    /// Некорректная структура KML.
    #[error("Invalid KML: {0}")]
    InvalidKml(String),
}

/// Парсит KML и возвращает список треков.
///
/// TODO: implement. KML — это XML с namespace,
/// структура Placemark/LineString/Point.
pub fn parse_kml(data: &str) -> Result<Vec<Track>, FormatError> {
    let _ = data;
    // TODO: implement full KML parsing via quick-xml.
    Ok(Vec::new())
}
