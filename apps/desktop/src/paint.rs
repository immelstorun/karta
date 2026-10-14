//! Отрисовка карты: тайлы, сетка координат.

use crate::app::KartaApp;
use crate::map_state::{MapState, TILE_SIZE};
use egui::{Color32, Pos2, Rect, Vec2};
use karta_core::coords::LatLon;
use karta_core::tiles::TileId;

/// Цвет фона карты и заглушки незагруженного тайла.
const PLACEHOLDER_COLOR: Color32 = Color32::from_rgb(50, 50, 55);

/// Цвет сетки координат.
const GRID_COLOR: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 60);

/// Отрисовывает карту на `ui`.
pub fn paint_map(ui: &mut egui::Ui, app: &mut KartaApp, show_grid: bool) {
    let viewport = ui.available_rect_before_wrap();
    let painter = ui.painter_at(viewport);

    // 1. Очистить фон (тёмно-серый).
    painter.rect_filled(viewport, 0.0, PLACEHOLDER_COLOR);

    // 2. Отрисовка видимых тайлов.
    let map_state = app.map_state.clone();
    let visible = compute_visible_tiles(&map_state, viewport);
    let scale = tile_scale(map_state.zoom);

    for tile in &visible {
        // Запрашиваем тайл (память → SAS.Planet кэш → сеть).
        app.tile_manager.request_tile(tile);

        // Прямоугольник тайла на экране.
        let rect = tile_screen_rect(&map_state, tile, viewport, scale);
        if !rect.intersects(viewport) {
            continue;
        }

        // Если текстура загружена — рисуем; иначе заглушка.
        match app.tiles_textures.get(tile) {
            Some(texture) => {
                painter.image(
                    texture.id(),
                    rect,
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            None => {
                painter.rect_filled(rect, 0.0, PLACEHOLDER_COLOR);
                // Индикатор загрузки в центре тайла.
                painter.circle_filled(rect.center(), 2.0, Color32::from_gray(100));
            }
        }
    }

    // 3. Отрисовка сетки координат (опционально).
    if show_grid {
        paint_grid(&painter, &map_state, viewport);
    }
}

/// Синхронизирует текстуры egui с загруженными тайлами.
pub fn sync_textures(app: &mut KartaApp, ctx: &egui::Context) {
    let tile_manager = &mut app.tile_manager;
    let tiles_textures = &mut app.tiles_textures;

    for tile in tile_manager.loaded_tile_ids() {
        if let std::collections::hash_map::Entry::Vacant(e) = tiles_textures.entry(tile) {
            if let Some(img) = tile_manager.get_tile(&tile) {
                let size = [img.width() as usize, img.height() as usize];
                let rgba = img.to_rgba8();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                let texture = ctx.load_texture(
                    format!("tile_{}_{}_{}", tile.zoom, tile.x, tile.y),
                    color_image,
                    egui::TextureOptions::default(),
                );
                e.insert(texture);
            }
        }
    }

    // Чистим текстуры для тайлов, выгруженных из памяти.
    tiles_textures.retain(|tile, _| tile_manager.is_loaded(tile));
}

/// Масштабный коэффициент тайла при дробном зуме:
/// экранная длина стороны тайла = TILE_SIZE * scale.
fn tile_scale(zoom: f64) -> f64 {
    2f64.powf(zoom - zoom.floor())
}

/// Вычисляет список видимых тайлов (на уровне `floor(zoom)`).
fn compute_visible_tiles(map_state: &MapState, viewport: Rect) -> Vec<TileId> {
    let zi = map_state.zoom.floor() as u8;
    let scale = tile_scale(map_state.zoom);
    let tiles_per_axis = 1u64 << zi;
    let tile_screen_size = TILE_SIZE * scale;

    // Границы видимой области в мировых пикселях (центр вьюпорта — центр карты).
    let (cx, cy) = map_state.lat_lon_to_world(&map_state.center);
    let half_w = (viewport.width() / 2.0) as f64;
    let half_h = (viewport.height() / 2.0) as f64;
    let min_wx = cx - half_w;
    let max_wx = cx + half_w;
    let min_wy = cy - half_h;
    let max_wy = cy + half_h;

    // Мировая координата p (в пикселях zoom `zi`) на экране занимает p * scale.
    let min_tx = ((min_wx / tile_screen_size).floor() as i64).max(0);
    let max_tx = ((max_wx / tile_screen_size).floor() as i64).min(tiles_per_axis as i64 - 1);
    let min_ty = ((min_wy / tile_screen_size).floor() as i64).max(0);
    let max_ty = ((max_wy / tile_screen_size).floor() as i64).min(tiles_per_axis as i64 - 1);

    let mut tiles = Vec::with_capacity(((max_tx - min_tx + 1) * (max_ty - min_ty + 1)) as usize);
    for ty in min_ty..=max_ty {
        for tx in min_tx..=max_tx {
            tiles.push(TileId {
                x: tx as u32,
                y: ty as u32,
                zoom: zi,
            });
        }
    }
    tiles
}

/// Вычисляет экранный прямоугольник тайла.
fn tile_screen_rect(map_state: &MapState, tile: &TileId, viewport: Rect, scale: f64) -> Rect {
    let tile_screen_size = TILE_SIZE * scale;
    let (cx, cy) = map_state.lat_lon_to_world(&map_state.center);
    let viewport_center = viewport.center();

    // Мировая позиция (в пикселях уровня тайла) левого верхнего угла тайла.
    let tile_wx = tile.x as f64 * tile_screen_size;
    let tile_wy = tile.y as f64 * tile_screen_size;

    let screen_x = viewport_center.x + (tile_wx - cx) as f32;
    let screen_y = viewport_center.y + (tile_wy - cy) as f32;

    Rect::from_min_size(
        Pos2::new(screen_x, screen_y),
        Vec2::new(tile_screen_size as f32, tile_screen_size as f32),
    )
}

/// Отрисовка сетки координат (меридианы и параллели).
fn paint_grid(painter: &egui::Painter, map_state: &MapState, viewport: Rect) {
    // Шаг сетки зависит от zoom: подбираем "красивый" шаг в градусах.
    let zoom = map_state.zoom;
    let step = if zoom < 3.0 {
        30.0
    } else if zoom < 5.0 {
        10.0
    } else if zoom < 7.0 {
        5.0
    } else if zoom < 9.0 {
        1.0
    } else if zoom < 11.0 {
        0.5
    } else if zoom < 13.0 {
        0.1
    } else if zoom < 15.0 {
        0.05
    } else {
        0.01
    };

    let vc = (viewport.center().x as f64, viewport.center().y as f64);

    // Географические границы видимой области.
    let top_left = map_state.screen_to_lat_lon((viewport.left() as f64, viewport.top() as f64), vc);
    let bottom_right =
        map_state.screen_to_lat_lon((viewport.right() as f64, viewport.bottom() as f64), vc);

    // Вертикальные линии (меридианы).
    let mut lon = (top_left.lon / step).floor() * step;
    while lon <= bottom_right.lon {
        let x = map_state.lat_lon_to_screen(&LatLon::new(0.0, lon), vc).0 as f32;
        if x >= viewport.left() && x <= viewport.right() {
            painter.line_segment(
                [
                    Pos2::new(x, viewport.top()),
                    Pos2::new(x, viewport.bottom()),
                ],
                egui::Stroke::new(1.0, GRID_COLOR),
            );
        }
        lon += step;
    }

    // Горизонтальные линии (параллели).
    let mut lat = (top_left.lat / step).floor() * step;
    while lat >= bottom_right.lat {
        let y = map_state.lat_lon_to_screen(&LatLon::new(lat, 0.0), vc).1 as f32;
        if y >= viewport.top() && y <= viewport.bottom() {
            painter.line_segment(
                [
                    Pos2::new(viewport.left(), y),
                    Pos2::new(viewport.right(), y),
                ],
                egui::Stroke::new(1.0, GRID_COLOR),
            );
        }
        lat -= step;
    }
}
