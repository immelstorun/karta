//! # karta-android-lib
//!
//! Нативная библиотека для Android-версии karta.
//! Предоставляет JNI-интерфейс к egui-приложению.
//!
//! На Android вызывается из `MainActivity.kt` через
//! `System.loadLibrary("karta")` + `karta_android_start()`.
//! На хосте (Linux/macOS/Windows) библиотека компилируется,
//! но весь android-код вырезается через `#[cfg]`.

use karta_core::coords::LatLon;

/// Точка входа, вызываемая из Java (MainActivity.kt) через JNI.
///
/// JNI-манглинг: метод `karta_android_start` класса
/// `com.karta.app.MainActivity` → символ
/// `Java_com_karta_app_MainActivity_karta_1android_1start`
/// (`_` в имени метода экранируется как `_1`).
///
/// Пока это заглушка: реальный запуск egui на Android требует
/// `android-activity` (GameActivity) и должен происходить из
/// `android_main` (см. ниже).
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_karta_app_MainActivity_karta_1android_1start(
    env: *mut std::ffi::c_void,
    class: *mut std::ffi::c_void,
) {
    let _ = (env, class);
    android_impl::android_main();
}

/// Точка входа Android NativeActivity / GameActivity.
///
/// eframe/winit на Android ожидают функцию `android_main`,
/// которая вызывается системой после создания нативной activity.
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn android_main(
    env: *mut std::ffi::c_void,
    klass: i32,
    activity: *mut std::ffi::c_void,
) {
    let _ = (env, klass, activity);
    android_impl::android_main();
}

// ── Заглушки для сборки на не-Android хостах ──────────────────────────

/// Не-Android хост: точка входа не нужна.
#[cfg(not(target_os = "android"))]
#[allow(dead_code)]
fn main() {}

/// Не-Android хост: реальный вход в egui-приложение.
///
/// На хосте (для отладки UI без Android) запускает то же окно,
/// что и desktop-приложение.
#[cfg(not(target_os = "android"))]
#[allow(dead_code)]
pub fn host_start() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0])
            .with_title("karta android preview (host)"),
        ..Default::default()
    };

    eframe::run_native(
        "karta",
        options,
        Box::new(|_cc| Ok(Box::new(KartaAndroidApp::default()))),
    )
}

// ── Общая часть приложения (и Android, и host) ────────────────────────

/// Упрощённое egui-приложение для Android-версии.
///
/// Полноценная версия должна переиспользовать `KartaApp` из
/// `apps/desktop`, вынесенный в общую библиотеку.
#[allow(dead_code)]
struct KartaAndroidApp {
    /// Центр карты (Москва по умолчанию).
    center: LatLon,
    /// Текущий zoom.
    zoom: f64,
}

impl Default for KartaAndroidApp {
    fn default() -> Self {
        Self {
            center: LatLon::new(55.7558, 37.6173),
            zoom: 10.0,
        }
    }
}

impl eframe::App for KartaAndroidApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("karta");
            ui.label(format!("Center: {:.4}, {:.4}", self.center.lat, self.center.lon));
            ui.label(format!("Zoom: {:.1}", self.zoom));
            ui.separator();
            ui.label("Android version — in development");
        });
    }
}

/// Android-специфичная инициализация.
#[cfg(target_os = "android")]
pub mod android_impl {
    /// Инициализация egui-приложения на Android.
    ///
    /// Это заглушка: реальный запуск требует настройки
    /// `android-activity` (GameActivity) через winit/eframe.
    pub fn android_main() {
        // Инициализация логирования в logcat.
        // В реальной реализации:
        // android_logger::init_once(android_logger::Config::default());
        tracing::info!("karta android app started");
    }
}
