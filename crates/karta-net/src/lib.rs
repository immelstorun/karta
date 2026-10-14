//! # karta-net
//!
//! Сетевой уровень karta: HTTP-клиент с connection pooling,
//! загрузка тайлов с retry и rate limiting.

pub mod http_client;
pub mod rate_limiter;
pub mod tile_fetcher;
