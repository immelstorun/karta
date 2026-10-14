//! Построение URL тайлов из шаблона.

use karta_core::tiles::TileId;

/// Шаблон URL тайлового сервера.
///
/// Поддерживаемые подстановки:
/// - `{s}` — поддомен (round-robin по координатам тайла);
/// - `{z}` — уровень зума;
/// - `{x}` — колонка тайла;
/// - `{y}` — строка тайла;
/// - `{q}` — Bing/Google QuadKey.
#[derive(Debug, Clone)]
pub struct UrlTemplate {
    /// Шаблон URL, например `https://tile.openstreetmap.org/{z}/{x}/{y}.png`.
    template: String,
    /// Список поддоменов для `{s}`.
    subdomains: Vec<String>,
}

impl UrlTemplate {
    /// Создаёт шаблон без поддоменов.
    pub fn new(template: &str) -> Self {
        Self {
            template: template.to_string(),
            subdomains: Vec::new(),
        }
    }

    /// Стандартный шаблон OpenStreetMap.
    pub fn osm_default() -> Self {
        Self::new("https://tile.openstreetmap.org/{z}/{x}/{y}.png")
    }

    /// Устанавливает поддомены для `{s}`.
    /// TODO(Phase 1): будет вызываться при загрузке TOML-слоя с поддоменами.
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn set_subdomains(&mut self, subdomains: &[&str]) {
        self.subdomains = subdomains.iter().map(|s| s.to_string()).collect();
    }

    /// Строит URL для заданного тайла.
    pub fn build_url(&self, tile: &TileId) -> String {
        let mut url = self.template.clone();
        url = url.replace("{z}", &tile.zoom.to_string());
        url = url.replace("{x}", &tile.x.to_string());
        url = url.replace("{y}", &tile.y.to_string());
        url = url.replace("{q}", &karta_core::tiles::quadkey(tile));

        if url.contains("{s}") {
            // Round-robin без состояния: индекс определяется координатами тайла.
            let n = self.subdomains.len().max(1);
            let idx = ((tile.x as usize) + (tile.y as usize) * 7) % n;
            let s = self
                .subdomains
                .get(idx)
                .cloned()
                .unwrap_or_else(|| "a".to_string());
            url = url.replace("{s}", &s);
        }

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url() {
        let template = UrlTemplate::osm_default();
        let tile = TileId {
            x: 1,
            y: 2,
            zoom: 3,
        };
        let url = template.build_url(&tile);
        assert_eq!(url, "https://tile.openstreetmap.org/3/1/2.png");
    }

    #[test]
    fn test_subdomains() {
        let mut template = UrlTemplate::new("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png");
        template.set_subdomains(&["a", "b", "c"]);
        let tile = TileId {
            x: 1,
            y: 2,
            zoom: 3,
        };
        let url = template.build_url(&tile);
        assert!(
            url.starts_with("https://a.tile.")
                || url.starts_with("https://b.tile.")
                || url.starts_with("https://c.tile.")
        );
    }

    #[test]
    fn test_quadkey_substitution() {
        let template = UrlTemplate::new("https://example.com/{q}.png");
        let tile = TileId {
            x: 3,
            y: 5,
            zoom: 3,
        };
        let url = template.build_url(&tile);
        assert_eq!(url, "https://example.com/213.png");
    }
}
