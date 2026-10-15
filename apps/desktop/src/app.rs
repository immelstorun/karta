//! Основное состояние приложения (impl eframe::App).

use crate::map_state::MapState;
use crate::paint::paint_map;
use crate::tile_manager::TileManager;
use crate::url_builder::UrlTemplate;
use karta_core::coords::LatLon;
use karta_core::tiles::TileId;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Описание картографического слоя.
#[derive(Debug, Clone)]
pub struct LayerInfo {
    /// Имя слоя (для кэша).
    pub name: String,
    /// Отображаемое название.
    pub title: String,
    /// Шаблон URL.
    pub url_template: UrlTemplate,
    /// Прозрачность слоя (0.0 — невидимый, 1.0 — полностью видимый).
    pub opacity: f32,
}

/// Главное окно приложения karta.
pub struct KartaApp {
    /// Менеджер тайлов: кэш + async-загрузка.
    pub tile_manager: TileManager,
    /// Состояние карты: центр, zoom.
    pub map_state: MapState,
    /// Загруженные текстуры тайлов для отрисовки.
    pub tiles_textures: HashMap<TileId, egui::TextureHandle>,
    /// Позиция курсора в географических координатах (для статусбара).
    pub cursor_lat_lon: Option<LatLon>,
    /// Показывать сетку координат.
    pub show_grid: bool,
    /// Показывать боковую панель.
    pub show_side_panel: bool,
    /// Счётчик кадров (для отладки производительности).
    pub frame_count: u64,
    /// Доступные слои.
    pub layers: Vec<LayerInfo>,
    /// Индекс активного слоя.
    pub active_layer: usize,
}

impl KartaApp {
    /// Создаёт приложение с указанным контекстом eframe.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Каталог кэша: XDG cache dir или временный каталог.
        let cache_dir: PathBuf = {
            let base = cache_base_dir();
            base.join("karta").join("tiles")
        };
        std::fs::create_dir_all(&cache_dir).expect("Не удалось создать каталог кэша");

        // Слои nakarte.me (TMS Y-flip для советских топокарт).
        let layers = vec![
            LayerInfo {
                name: "ggc500".to_string(),
                title: "ГГЦ 1:500000".to_string(),
                url_template: {
                    let mut t = UrlTemplate::new("https://tiles.nakarte.me/ggc500/{z}/{x}/{y}");
                    t.set_tms_y_flip(true);
                    t
                },
                opacity: 1.0,
            },
            LayerInfo {
                name: "ggc250".to_string(),
                title: "ГГЦ 1:250000".to_string(),
                url_template: {
                    let mut t = UrlTemplate::new("https://tiles.nakarte.me/ggc250/{z}/{x}/{y}");
                    t.set_tms_y_flip(true);
                    t
                },
                opacity: 1.0,
            },
            LayerInfo {
                name: "ggc2000".to_string(),
                title: "ГГЦ 1:200000".to_string(),
                url_template: {
                    let mut t = UrlTemplate::new("https://tiles.nakarte.me/ggc2000/{z}/{x}/{y}");
                    t.set_tms_y_flip(true);
                    t
                },
                opacity: 1.0,
            },
            LayerInfo {
                name: "topomapper".to_string(),
                title: "TopoMapper".to_string(),
                url_template: UrlTemplate::new("https://tiles.nakarte.me/topomapper/{z}/{x}/{y}"),
                opacity: 1.0,
            },
            LayerInfo {
                name: "osm".to_string(),
                title: "OpenStreetMap".to_string(),
                url_template: UrlTemplate::osm_default(),
                opacity: 1.0,
            },
        ];

        let active_layer = 0;
        let tile_manager = TileManager::new(
            cache_dir,
            layers[active_layer].url_template.clone(),
            &layers[active_layer].name,
        )
        .expect("Не удалось создать TileManager");

        Self {
            tile_manager,
            map_state: MapState::new(),
            tiles_textures: HashMap::new(),
            cursor_lat_lon: None,
            show_grid: false,
            show_side_panel: true,
            frame_count: 0,
            layers,
            active_layer,
        }
    }

    /// Переключает активный слой.
    pub fn switch_layer(&mut self, index: usize) {
        if index >= self.layers.len() || index == self.active_layer {
            return;
        }
        self.active_layer = index;
        let layer = &self.layers[index];
        self.tile_manager
            .set_url_template(layer.url_template.clone());
        self.tile_manager.set_layer_name(&layer.name);
        // Очищаем загруженные тайлы — они от другого слоя.
        self.tiles_textures.clear();
    }

    /// Обрабатывает ввод: мышь (drag → pan, scroll → zoom) и клавиатуру.
    fn handle_input(&mut self, ui: &mut egui::Ui, viewport: egui::Rect) {
        let vc = (viewport.center().x as f64, viewport.center().y as f64);

        // ── Колесо мыши → zoom к точке под курсором ──────────────────
        let scroll = ui.input(|i| i.smooth_scroll_delta().y);
        if scroll.abs() > f32::EPSILON {
            if let Some(hover) = ui.input(|i| i.pointer.hover_pos()) {
                let anchor = (hover.x as f64, hover.y as f64);
                let dz = (scroll * 0.01).clamp(-1.0, 1.0);
                self.map_state.zoom_at(anchor, vc, dz as f64);
            }
        }

        // ── Drag → pan ────────────────────────────────────────────────
        let pointer_down = ui.input(|i| i.pointer.primary_down());
        let pointer_delta = ui.input(|i| i.pointer.delta());
        if pointer_down {
            self.map_state
                .pan(pointer_delta.x as f64, pointer_delta.y as f64);
        }

        // ── Клавиатура ────────────────────────────────────────────────
        // +/= → zoom in, -/_ → zoom out, 0 → reset, стрелки → pan.
        let mut text_events: Vec<String> = Vec::new();
        ui.input(|i| {
            for event in &i.events {
                if let egui::Event::Text(t) = event {
                    text_events.push(t.clone());
                }
            }
        });
        for t in text_events {
            match t.as_str() {
                "+" | "=" => self.map_state.zoom_in(),
                "-" | "_" => self.map_state.zoom_out(),
                "0" => self.map_state.reset(),
                _ => {}
            }
        }

        let (arrow_left, arrow_right, arrow_up, arrow_down) = ui.input(|i| {
            (
                i.key_down(egui::Key::ArrowLeft),
                i.key_down(egui::Key::ArrowRight),
                i.key_down(egui::Key::ArrowUp),
                i.key_down(egui::Key::ArrowDown),
            )
        });
        let pan_step = 50.0;
        if arrow_left {
            self.map_state.pan(pan_step, 0.0);
        }
        if arrow_right {
            self.map_state.pan(-pan_step, 0.0);
        }
        if arrow_up {
            self.map_state.pan(0.0, pan_step);
        }
        if arrow_down {
            self.map_state.pan(0.0, -pan_step);
        }
    }
}

impl eframe::App for KartaApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.frame_count += 1;

        // ── Top panel: меню ────────────────────────────────────────────
        egui::Panel::top("top_panel").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Zoom in (+)").clicked() {
                        self.map_state.zoom_in();
                    }
                    if ui.button("Zoom out (-)").clicked() {
                        self.map_state.zoom_out();
                    }
                    if ui.button("Reset (0)").clicked() {
                        self.map_state.reset();
                    }
                    ui.separator();
                    ui.checkbox(&mut self.show_grid, "Show grid");
                    ui.checkbox(&mut self.show_side_panel, "Show side panel");
                });
                ui.menu_button("Layers", |ui| {
                    let layers = self.layers.clone();
                    for (i, layer) in layers.iter().enumerate() {
                        if ui
                            .radio_value(&mut self.active_layer, i, &layer.title)
                            .clicked()
                        {
                            self.switch_layer(i);
                        }
                    }
                    ui.separator();
                    // Прозрачность активного слоя.
                    if let Some(layer) = self.layers.get_mut(self.active_layer) {
                        ui.label("Прозрачность:");
                        ui.add(egui::Slider::new(&mut layer.opacity, 0.0..=1.0));
                    }
                });
                ui.menu_button("Help", |ui| {
                    ui.label("karta — карта для туристов");
                    ui.label("Версия: 0.0.1");
                    ui.separator();
                    ui.hyperlink("https://github.com/example/karta");
                });
            });
        });

        // ── Bottom panel: статусбар ────────────────────────────────────
        egui::Panel::bottom("bottom_panel").show(ui, |ui| {
            ui.set_min_height(24.0);
            ui.horizontal(|ui| {
                ui.label(format!("Zoom: {:.1}", self.map_state.zoom));
                ui.separator();
                match self.cursor_lat_lon {
                    Some(ll) => {
                        ui.label(format!("Широта: {:.5}", ll.lat));
                        ui.label(format!("Долгота: {:.5}", ll.lon));
                    }
                    None => {
                        ui.label("Курсор вне карты");
                    }
                }
                ui.separator();
                ui.label(format!(
                    "Тайлы: {} загружено, {} в процессе",
                    self.tile_manager.loaded_count(),
                    self.tile_manager.pending_count()
                ));
                ui.separator();
                // Источник тайлов и attribution.
                if let Some(layer) = self.layers.get(self.active_layer) {
                    ui.label(format!("© {}", layer.title));
                }
            });
        });

        // ── Side panel: информация ────────────────────────────────────
        if self.show_side_panel {
            egui::Panel::right("side_panel")
                .default_size(220.0)
                .show(ui, |ui| {
                    ui.heading("karta");
                    ui.separator();

                    ui.label("Информация о карте");
                    ui.label(format!(
                        "Центр: {:.4}, {:.4}",
                        self.map_state.center.lat, self.map_state.center.lon
                    ));
                    ui.label(format!("Zoom: {:.2}", self.map_state.zoom));
                    ui.separator();

                    ui.label("Статистика");
                    ui.label(format!(
                        "Загружено тайлов: {}",
                        self.tile_manager.loaded_count()
                    ));
                    ui.label(format!("В процессе: {}", self.tile_manager.pending_count()));
                    ui.separator();

                    ui.label("Управление");
                    ui.label("• ЛКМ + drag — панорамирование");
                    ui.label("• Колесо мыши — zoom");
                    ui.label("• +/− — zoom in/out");
                    ui.label("• 0 — сброс вида");
                    ui.label("• Стрелки — pan");
                });
        }

        // ── Central panel: карта ──────────────────────────────────────
        egui::CentralPanel::default().show(ui, |ui| {
            let viewport = ui.available_rect_before_wrap();

            // Обрабатываем ввод до отрисовки.
            self.handle_input(ui, viewport);

            // Обновляем координаты курсора для статусбара.
            if let Some(hover) = ui.input(|i| i.pointer.hover_pos()) {
                if viewport.contains(hover) {
                    let vc = (viewport.center().x as f64, viewport.center().y as f64);
                    let mp = (hover.x as f64, hover.y as f64);
                    self.cursor_lat_lon = Some(self.map_state.screen_to_lat_lon(mp, vc));
                }
            }

            // Обрабатываем ответы фоновых задач загрузки.
            self.tile_manager.process_responses();

            // Синхронизируем текстуры egui с загруженными тайлами.
            crate::paint::sync_textures(self, ui.ctx());

            // Отрисовка карты.
            let show_grid = self.show_grid;
            paint_map(ui, self, show_grid);
        });

        // Плавная перерисовка для загрузки тайлов.
        ui.ctx().request_repaint_after(Duration::from_millis(100));
    }
}

/// Возвращает базовый каталог для кэша (XDG-совместимый).
fn cache_base_dir() -> PathBuf {
    // Пробуем XDG_CACHE_HOME, затем ~/.cache, затем /tmp.
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg);
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        let p = PathBuf::from(home).join(".cache");
        if p.exists() {
            return p;
        }
    }
    PathBuf::from("/tmp")
}
