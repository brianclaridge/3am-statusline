use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const GEOCODE_CACHE: &str = ".data/statusline/geocode.json";

#[derive(Serialize)]
struct WeatherOutput {
    emoji: String,
    temp: String,
    condition: String,
}

#[derive(Serialize, Deserialize)]
struct GeoCache {
    zip: String,
    lat: f64,
    lon: f64,
}

#[derive(Deserialize)]
struct ZipResponse {
    places: Vec<ZipPlace>,
}

#[derive(Deserialize)]
struct ZipPlace {
    latitude: String,
    longitude: String,
}

#[derive(Deserialize)]
struct MeteoResponse {
    current: MeteoCurrent,
}

#[derive(Deserialize)]
struct MeteoCurrent {
    temperature_2m: f64,
    weather_code: u32,
    is_day: u8,
}

fn resolve_coords(zip: &str) -> Result<(f64, f64)> {
    let cache_path = Path::new(GEOCODE_CACHE);

    // Check cache
    if cache_path.exists() {
        if let Ok(data) = std::fs::read_to_string(cache_path) {
            if let Ok(cached) = serde_json::from_str::<GeoCache>(&data) {
                if cached.zip == zip {
                    return Ok((cached.lat, cached.lon));
                }
            }
        }
    }

    // Fetch from Zippopotam.us
    let url = format!("https://api.zippopotam.us/us/{zip}");
    let body: String = ureq::get(&url)
        .call()
        .context("geocode request failed")?
        .body_mut()
        .read_to_string()
        .context("reading geocode response")?;
    let resp: ZipResponse = serde_json::from_str(&body).context("parsing geocode JSON")?;
    let place = resp.places.first().context("no places in geocode response")?;
    let lat: f64 = place.latitude.parse().context("parsing latitude")?;
    let lon: f64 = place.longitude.parse().context("parsing longitude")?;

    // Cache permanently
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let cache = GeoCache { zip: zip.to_string(), lat, lon };
    let _ = std::fs::write(cache_path, serde_json::to_string(&cache)?);

    Ok((lat, lon))
}

fn wmo_emoji(code: u32, is_day: bool) -> (&'static str, &'static str) {
    match code {
        0 if is_day => ("\u{2600}\u{fe0f}", "clear"),       // ☀️
        0 => ("\u{1f319}", "clear"),                         // 🌙
        1..=3 if is_day => ("\u{26c5}", "cloudy"),           // ⛅
        1..=3 => ("\u{1f319}", "cloudy"),                    // 🌙
        45 | 48 => ("\u{1f32b}\u{fe0f}", "fog"),             // 🌫️
        51..=57 if is_day => ("\u{1f326}\u{fe0f}", "drizzle"), // 🌦️
        51..=57 => ("\u{1f327}\u{fe0f}", "drizzle"),         // 🌧️
        61..=67 => ("\u{1f327}\u{fe0f}", "rain"),            // 🌧️
        71..=77 => ("\u{1f328}\u{fe0f}", "snow"),            // 🌨️
        80..=82 if is_day => ("\u{1f326}\u{fe0f}", "showers"), // 🌦️
        80..=82 => ("\u{1f327}\u{fe0f}", "showers"),         // 🌧️
        85 | 86 => ("\u{1f328}\u{fe0f}", "snow showers"),    // 🌨️
        95..=99 => ("\u{26c8}\u{fe0f}", "thunderstorm"),     // ⛈️
        _ => ("?", "unknown"),
    }
}

fn gather(zip: &str) -> Result<WeatherOutput> {
    let (lat, lon) = resolve_coords(zip)?;

    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current=temperature_2m,weather_code,is_day&temperature_unit=fahrenheit"
    );
    let body: String = ureq::get(&url)
        .call()
        .context("weather request failed")?
        .body_mut()
        .read_to_string()
        .context("reading weather response")?;
    let resp: MeteoResponse = serde_json::from_str(&body).context("parsing weather JSON")?;

    let is_day = resp.current.is_day == 1;
    let (emoji, condition) = wmo_emoji(resp.current.weather_code, is_day);

    Ok(WeatherOutput {
        emoji: emoji.to_string(),
        temp: format!("{:.0}\u{00b0}F", resp.current.temperature_2m),
        condition: condition.to_string(),
    })
}

pub fn run(zip: &str) -> Result<()> {
    let data = gather(zip)?;
    println!("{}", serde_json::to_string(&data)?);
    Ok(())
}
