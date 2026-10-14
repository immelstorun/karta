//! GPX (GPS Exchange Format) — импорт и экспорт треков и waypoints.

use karta_core::vector::{Track, Waypoint};

/// Ошибка разбора/экспорта GPX.
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    /// Ошибка XML-парсинга.
    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::Error),

    /// Некорректная структура GPX.
    #[error("Invalid GPX: {0}")]
    InvalidGpx(String),
}

/// Парсит GPX 1.1 и возвращает список треков.
///
/// Каждый `<trk>` содержит один или несколько `<trkseg>`;
/// все сегменты объединяются в один трек.
pub fn parse_gpx(data: &str) -> Result<Vec<Track>, FormatError> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(data);
    reader.config_mut().trim_text(true);

    let mut tracks: Vec<Track> = Vec::new();
    let mut current_track: Option<Track> = None;
    let mut current_name: Option<String> = None;
    let mut in_trk = false;
    let mut buf = Vec::new();
    let mut text_buf = String::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "trk" {
                    in_trk = true;
                    current_track = Some(Track {
                        name: String::new(),
                        points: Vec::new(),
                    });
                } else if name == "trkpt" {
                    let lat = e
                        .attributes()
                        .filter_map(|a| a.ok())
                        .find(|a| a.key.as_ref() == b"lat")
                        .and_then(|a| String::from_utf8_lossy(&a.value).parse::<f64>().ok())
                        .ok_or_else(|| {
                            FormatError::InvalidGpx("trkpt missing lat attribute".to_string())
                        })?;
                    let lon = e
                        .attributes()
                        .filter_map(|a| a.ok())
                        .find(|a| a.key.as_ref() == b"lon")
                        .and_then(|a| String::from_utf8_lossy(&a.value).parse::<f64>().ok())
                        .ok_or_else(|| {
                            FormatError::InvalidGpx("trkpt missing lon attribute".to_string())
                        })?;
                    if let Some(track) = current_track.as_mut() {
                        track.points.push(karta_core::vector::TrackPoint {
                            lat_lon: karta_core::coords::LatLon::new(lat, lon),
                            elevation: None,
                            time: None,
                        });
                    }
                } else if name == "name" {
                    text_buf.clear();
                }
            }
            Event::Text(ref t) => {
                if in_trk {
                    text_buf.push_str(&String::from_utf8_lossy(t.as_ref()));
                }
            }
            Event::End(ref e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "name" if in_trk => {
                        current_name = Some(text_buf.trim().to_string());
                        text_buf.clear();
                    }
                    "trk" => {
                        in_trk = false;
                        if let Some(mut track) = current_track.take() {
                            track.name =
                                current_name.take().unwrap_or_else(|| "unnamed".to_string());
                            tracks.push(track);
                        }
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(tracks)
}

/// Парсит GPX 1.1 и возвращает список waypoints (`<wpt>`).
pub fn parse_gpx_waypoints(data: &str) -> Result<Vec<Waypoint>, FormatError> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(data);
    reader.config_mut().trim_text(true);

    let mut waypoints: Vec<Waypoint> = Vec::new();
    let mut in_wpt = false;
    let mut buf = Vec::new();
    let mut text_buf = String::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "wpt" {
                    in_wpt = true;
                    let lat = e
                        .attributes()
                        .filter_map(|a| a.ok())
                        .find(|a| a.key.as_ref() == b"lat")
                        .and_then(|a| String::from_utf8_lossy(&a.value).parse::<f64>().ok())
                        .ok_or_else(|| {
                            FormatError::InvalidGpx("wpt missing lat attribute".to_string())
                        })?;
                    let lon = e
                        .attributes()
                        .filter_map(|a| a.ok())
                        .find(|a| a.key.as_ref() == b"lon")
                        .and_then(|a| String::from_utf8_lossy(&a.value).parse::<f64>().ok())
                        .ok_or_else(|| {
                            FormatError::InvalidGpx("wpt missing lon attribute".to_string())
                        })?;
                    waypoints.push(Waypoint {
                        name: String::new(),
                        lat_lon: karta_core::coords::LatLon::new(lat, lon),
                        description: None,
                    });
                } else if in_wpt && (name == "name" || name == "desc") {
                    text_buf.clear();
                }
            }
            Event::Text(ref t) => {
                if in_wpt {
                    text_buf.push_str(&String::from_utf8_lossy(t.as_ref()));
                }
            }
            Event::End(ref e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "wpt" {
                    in_wpt = false;
                } else if in_wpt {
                    if let Some(waypoint) = waypoints.last_mut() {
                        match name.as_str() {
                            "name" => waypoint.name = text_buf.trim().to_string(),
                            "desc" => waypoint.description = Some(text_buf.trim().to_string()),
                            _ => {}
                        }
                    }
                }
                text_buf.clear();
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(waypoints)
}

/// Экспортирует треки и waypoints в GPX 1.1.
pub fn export_gpx(tracks: &[Track], waypoints: &[Waypoint]) -> Result<String, FormatError> {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(
        "<gpx version=\"1.1\" creator=\"karta\" xmlns=\"http://www.topografix.com/GPX/1/1\">\n",
    );

    for track in tracks {
        out.push_str("  <trk>\n");
        out.push_str(&format!("    <name>{}</name>\n", escape_xml(&track.name)));
        for point in &track.points {
            out.push_str(&format!(
                "    <trkpt lat=\"{:.7}\" lon=\"{:.7}\">\n",
                point.lat_lon.lat, point.lat_lon.lon
            ));
            if let Some(elevation) = point.elevation {
                out.push_str(&format!("      <ele>{:.2}</ele>\n", elevation));
            }
            if let Some(time) = point.time {
                out.push_str(&format!("      <time>{}</time>\n", time.to_rfc3339()));
            }
            out.push_str("    </trkpt>\n");
        }
        out.push_str("  </trk>\n");
    }

    for waypoint in waypoints {
        out.push_str(&format!(
            "  <wpt lat=\"{:.7}\" lon=\"{:.7}\">\n",
            waypoint.lat_lon.lat, waypoint.lat_lon.lon
        ));
        if !waypoint.name.is_empty() {
            out.push_str(&format!(
                "    <name>{}</name>\n",
                escape_xml(&waypoint.name)
            ));
        }
        if let Some(description) = &waypoint.description {
            out.push_str(&format!("    <desc>{}</desc>\n", escape_xml(description)));
        }
        out.push_str("  </wpt>\n");
    }

    out.push_str("</gpx>\n");
    Ok(out)
}

/// Экранирует спецсимволы XML.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_GPX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="test" xmlns="http://www.topografix.com/GPX/1/1">
  <wpt lat="55.7558" lon="37.6173">
    <name>Moscow</name>
    <desc>Red Square</desc>
  </wpt>
  <trk>
    <name>Test Track</name>
    <trkseg>
      <trkpt lat="55.7558" lon="37.6173"><ele>150.0</ele></trkpt>
      <trkpt lat="55.7560" lon="37.6175"><ele>151.0</ele></trkpt>
    </trkseg>
  </trk>
</gpx>"#;

    #[test]
    fn test_parse_gpx() {
        let tracks = parse_gpx(SAMPLE_GPX).expect("parse failed");
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].name, "Test Track");
        assert_eq!(tracks[0].points.len(), 2);
        assert!((tracks[0].points[0].lat_lon.lat - 55.7558).abs() < 1e-6);
        assert!((tracks[0].points[0].lat_lon.lon - 37.6173).abs() < 1e-6);
    }

    #[test]
    fn test_parse_gpx_waypoints() {
        let waypoints = parse_gpx_waypoints(SAMPLE_GPX).expect("parse failed");
        assert_eq!(waypoints.len(), 1);
        assert_eq!(waypoints[0].name, "Moscow");
        assert_eq!(waypoints[0].description.as_deref(), Some("Red Square"));
        assert!((waypoints[0].lat_lon.lat - 55.7558).abs() < 1e-6);
    }

    #[test]
    fn test_export_gpx_roundtrip() {
        let tracks = parse_gpx(SAMPLE_GPX).unwrap();
        let waypoints = parse_gpx_waypoints(SAMPLE_GPX).unwrap();
        let exported = export_gpx(&tracks, &waypoints).expect("export failed");

        let reparsed_tracks = parse_gpx(&exported).unwrap();
        let reparsed_waypoints = parse_gpx_waypoints(&exported).unwrap();

        assert_eq!(reparsed_tracks.len(), 1);
        assert_eq!(reparsed_tracks[0].name, "Test Track");
        assert_eq!(reparsed_tracks[0].points.len(), 2);
        assert_eq!(reparsed_waypoints.len(), 1);
        assert_eq!(reparsed_waypoints[0].name, "Moscow");
    }

    #[test]
    fn test_parse_gpx_invalid() {
        let result =
            parse_gpx("<gpx><trk><trkpt lat=\"notanumber\" lon=\"0\"></trkpt></trk></gpx>");
        assert!(result.is_err());
    }
}
