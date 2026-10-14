//! Декларативные слои (TOML-манифесты).
//!
//! Слой можно описать декларативно в TOML-файле:
//!
//! ```toml
//! [layer]
//! id = "osm-standard"
//! name = "OpenStreetMap Standard"
//! url_template = "https://tile.openstreetmap.org/{z}/{x}/{y}.png"
//! max_zoom = 19
//! min_zoom = 0
//! tile_size = 256
//! attribution = "© OpenStreetMap contributors"
//! tms_y_flip = false
//! ```
//!
//! TODO: поддержка вложенных слоёв, conditional layers,
//! пользовательских источников (MBTiles, SAS.Planet cache).

use serde::{Deserialize, Serialize};

/// Описание слоя в TOML-манифесте.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerSpec {
    /// Уникальный идентификатор слоя
    pub id: String,
    /// Человекочитаемое имя слоя
    pub name: String,
    /// URL-шаблон тайла: `https://tile.example.org/{z}/{x}/{y}.png`
    pub url_template: String,
    /// Максимальный zoom слоя
    pub max_zoom: u8,
    /// Минимальный zoom слоя
    pub min_zoom: u8,
    /// Размер тайла слоя в пикселях (обычно 256)
    pub tile_size: u32,
    /// Атрибуция слоя (copyright notice)
    pub attribution: Option<String>,
    /// Использовать ли TMS y-flip (некоторые провайдеры)
    pub tms_y_flip: bool,
}

/// Структура TOML-манифеста слоя.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerManifest {
    /// Версия формата манифеста
    pub version: u32,
    /// Описание слоя
    pub layer: LayerSpec,
}

/// Ошибка парсинга TOML-манифеста.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// TOML syntax error
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
}

/// Парсит TOML-манифест слоя.
///
/// # Examples
///
/// ```
/// use karta_plugin_host::declarative_layers::parse_layer_manifest;
///
/// let toml_str = r#"
/// version = 1
///
/// [layer]
/// id = "osm"
/// name = "OSM"
/// url_template = "https://tile.openstreetmap.org/{z}/{x}/{y}.png"
/// max_zoom = 19
/// min_zoom = 0
/// tile_size = 256
/// tms_y_flip = false
/// "#;
/// let manifest = parse_layer_manifest(toml_str);
/// assert!(manifest.is_ok());
/// ```
pub fn parse_layer_manifest(toml_str: &str) -> Result<LayerManifest, Error> {
    let manifest: LayerManifest = toml::from_str(toml_str)?;
    Ok(manifest)
}

/// Строит URL тайла из спецификации слоя.
///
/// Поддерживает плейсхолдеры `{z}`, `{x}`, `{y}` в `url_template`.
/// Если `tms_y_flip = true`, y-координата инвертируется (TMS scheme).
pub fn build_tile_url(spec: &LayerSpec, tile: &karta_core::tiles::TileId) -> String {
    let y = if spec.tms_y_flip {
        ((1u64 << tile.zoom) - 1) - tile.y as u64
    } else {
        tile.y as u64
    };

    spec.url_template
        .replace("{z}", &tile.zoom.to_string())
        .replace("{x}", &tile.x.to_string())
        .replace("{y}", &y.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use karta_core::tiles::TileId;

    fn test_spec() -> LayerSpec {
        LayerSpec {
            id: "test".to_string(),
            name: "Test Layer".to_string(),
            url_template: "https://tile.example.org/{z}/{x}/{y}.png".to_string(),
            max_zoom: 18,
            min_zoom: 0,
            tile_size: 256,
            attribution: Some("© Test".to_string()),
            tms_y_flip: false,
        }
    }

    #[test]
    fn test_parse_layer_manifest() {
        let toml_str = r#"
            version = 1

            [layer]
            id = "osm"
            name = "OSM Standard"
            url_template = "https://tile.openstreetmap.org/{z}/{x}/{y}.png"
            max_zoom = 19
            min_zoom = 0
            tile_size = 256
            tms_y_flip = false
        "#;
        let manifest = parse_layer_manifest(toml_str).expect("manifest should parse");
        assert_eq!(manifest.version, 1);
        assert_eq!(manifest.layer.id, "osm");
        assert_eq!(manifest.layer.tile_size, 256);
        assert!(!manifest.layer.tms_y_flip);
    }

    #[test]
    fn test_parse_layer_manifest_invalid() {
        let toml_str = "not valid toml {{{";
        assert!(parse_layer_manifest(toml_str).is_err());
    }

    #[test]
    fn test_build_tile_url() {
        let spec = test_spec();
        let tile = TileId {
            x: 3,
            y: 5,
            zoom: 7,
        };
        let url = build_tile_url(&spec, &tile);
        assert_eq!(url, "https://tile.example.org/7/3/5.png");
    }

    #[test]
    fn test_build_tile_url_tms_flip() {
        let mut spec = test_spec();
        spec.tms_y_flip = true;
        let tile = TileId {
            x: 3,
            y: 5,
            zoom: 7,
        };
        let url = build_tile_url(&spec, &tile);
        // TMS y = (2^7 - 1) - 5 = 127 - 5 = 122
        assert_eq!(url, "https://tile.example.org/7/3/122.png");
    }
}
