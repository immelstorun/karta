//! # karta-cli
//!
//! CLI-утилиты для работы с karta:
//! - Загрузка тайлов
//! - Управление кэшем
//! - Конвертация между форматами кэшей

use clap::{Args, Parser, Subcommand};
use karta_cache::{SasCache, TileCache};
use karta_core::tiles::{lat_lon_to_tile, TileId};
use karta_net::http_client::HttpClientConfig;
use karta_net::rate_limiter::RateLimiterConfig;
use karta_net::tile_fetcher::{TileFetchRequest, TileFetcher};
use std::path::Path;
use std::sync::Arc;

/// CLI для работы с тайлами и кэшами karta.
#[derive(Parser)]
#[command(name = "karta-cli", version, about = "karta CLI tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Download map tiles for a region
    Download(DownloadArgs),
    /// Show cache statistics
    Cache(CacheArgs),
    /// Convert between tile cache formats
    Convert(ConvertArgs),
    /// List available layers
    Layers(LayersArgs),
}

#[derive(Args)]
struct DownloadArgs {
    /// Zoom level
    #[arg(short, long, default_value = "10")]
    zoom: u8,
    /// Center latitude
    #[arg(short = 'a', long, default_value = "55.7558")]
    lat: f64,
    /// Center longitude
    #[arg(short = 'b', long, default_value = "37.6173")]
    lon: f64,
    /// Radius in tiles
    #[arg(short, long, default_value = "3")]
    radius: u32,
    /// Cache directory
    #[arg(short, long, default_value = "./cache")]
    cache_dir: String,
}

#[derive(Args)]
struct CacheArgs {
    /// Cache directory
    #[arg(short, long, default_value = "./cache")]
    cache_dir: String,
}

#[derive(Args)]
struct ConvertArgs {
    /// Source cache directory
    #[arg(short, long)]
    from: String,
    /// Target cache format (mbtiles, sas)
    #[arg(short, long)]
    to: String,
}

#[derive(Args)]
struct LayersArgs {
    /// Filter layers by name (substring)
    #[arg(short, long)]
    filter: Option<String>,
}

/// Вычисляет все тайлы в квадрате радиуса вокруг центрального тайла.
fn tiles_in_radius(lat: f64, lon: f64, zoom: u8, radius: u32) -> Vec<TileId> {
    let center = lat_lon_to_tile(lat, lon, zoom);
    let mut tiles = Vec::new();
    for dx in -(radius as i64)..=(radius as i64) {
        for dy in -(radius as i64)..=(radius as i64) {
            // Проверяем, что мы внутри круга (евклидово расстояние)
            if (dx * dx + dy * dy) <= (radius * radius) as i64 {
                let x = center.x as i64 + dx;
                let y = center.y as i64 + dy;
                if x >= 0 && y >= 0 && x < (1i64 << zoom) && y < (1i64 << zoom) {
                    tiles.push(TileId {
                        x: x as u32,
                        y: y as u32,
                        zoom,
                    });
                }
            }
        }
    }
    tiles
}

/// Команда загрузки тайлов в кэш.
///
/// Алгоритм:
/// 1. Вычисляем все тайлы в радиусе вокруг центра
/// 2. Проверяем кэш, загружаем только отсутствующие
/// 3. Показываем прогресс
async fn cmd_download(args: &DownloadArgs) -> anyhow::Result<()> {
    let tiles = tiles_in_radius(args.lat, args.lon, args.zoom, args.radius);

    println!(
        "Downloading {} tiles (zoom={}, lat={}, lon={}, radius={})",
        tiles.len(),
        args.zoom,
        args.lat,
        args.lon,
        args.radius
    );

    let cache = SasCache::new(&args.cache_dir);
    let mut downloaded = 0usize;
    let mut cached = 0usize;

    // TODO: configure tile URL template and layer name via CLI arguments
    let layer_name = "osm";
    let url_template = "https://tile.openstreetmap.org/{z}/{x}/{y}.png";

    let http_client = karta_net::http_client::HttpClient::new(HttpClientConfig::default())?;
    let rate_limiter = karta_net::rate_limiter::RateLimiter::new(RateLimiterConfig::default());
    let fetcher = TileFetcher::new(Arc::new(http_client), Arc::new(rate_limiter));

    for (idx, tile) in tiles.iter().enumerate() {
        let progress = (idx + 1) * 100 / tiles.len();
        print!(
            "\r[{:3}%] {}/{} tiles processed",
            progress,
            idx + 1,
            tiles.len()
        );
        use std::io::Write;
        std::io::stdout().flush()?;

        // Check cache first
        match cache.get_tile(layer_name, tile).await {
            Ok(Some(_)) => {
                cached += 1;
                continue;
            }
            Ok(None) => {
                // Tile not in cache, download it
                let url = url_template
                    .replace("{z}", &tile.zoom.to_string())
                    .replace("{x}", &tile.x.to_string())
                    .replace("{y}", &tile.y.to_string());

                let request = TileFetchRequest {
                    url,
                    headers: vec![],
                    retry_count: 3,
                    retry_delay_ms: 500,
                };
                match fetcher.fetch_tile(&request).await {
                    Ok(data) => {
                        cache.put_tile(layer_name, tile, &data).await?;
                        downloaded += 1;
                    }
                    Err(e) => {
                        eprintln!("\nFailed to download tile {:?}: {}", tile, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("\nCache error for tile {:?}: {}", tile, e);
            }
        }
    }

    println!(
        "\nDone: {} tiles downloaded, {} already in cache",
        downloaded, cached
    );
    Ok(())
}

/// Команда показа статистики кэша.
async fn cmd_cache(args: &CacheArgs) -> anyhow::Result<()> {
    let cache_dir = Path::new(&args.cache_dir);

    if !cache_dir.exists() {
        println!("Cache directory '{}' does not exist", args.cache_dir);
        return Ok(());
    }

    // TODO: show detailed cache statistics (per-layer, per-zoom, total size, tile counts)
    println!("Cache statistics for '{}':", args.cache_dir);

    let mut total_size: u64 = 0;
    let mut total_files: u64 = 0;

    fn visit_dir(dir: &Path, total_size: &mut u64, total_files: &mut u64) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    visit_dir(&path, total_size, total_files);
                } else if let Ok(meta) = entry.metadata() {
                    *total_size += meta.len();
                    *total_files += 1;
                }
            }
        }
    }

    visit_dir(cache_dir, &mut total_size, &mut total_files);

    println!("  Total files: {}", total_files);
    println!("  Total size: {} MB", total_size / (1024 * 1024));
    Ok(())
}

/// Команда конвертации кэшей.
async fn cmd_convert(args: &ConvertArgs) -> anyhow::Result<()> {
    // TODO: implement cache conversion (SAS.Planet -> MBTiles, etc.)
    println!(
        "Converting cache from '{}' to format '{}' (not implemented yet)",
        args.from, args.to
    );
    Ok(())
}

/// Команда показа доступных слоёв.
async fn cmd_layers(args: &LayersArgs) -> anyhow::Result<()> {
    // TODO: implement layer discovery (from config directory, built-in layers, etc.)
    let layers = vec![
        ("osm", "OpenStreetMap Standard"),
        ("osm-humanitarian", "OpenStreetMap Humanitarian"),
        ("opentopomap", "OpenTopoMap"),
    ];

    println!("Available layers:");
    for (id, name) in layers {
        if let Some(filter) = &args.filter {
            if !id.contains(filter) && !name.contains(filter) {
                continue;
            }
        }
        println!("  {:<20} {}", id, name);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Download(args) => cmd_download(args).await,
        Commands::Cache(args) => cmd_cache(args).await,
        Commands::Convert(args) => cmd_convert(args).await,
        Commands::Layers(args) => cmd_layers(args).await,
    }
}
