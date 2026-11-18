//! wttr.in adapter matching the original WeatherService/ scripts/weather.py.

use serde_json::Value;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeatherReport {
    pub place: String,
    pub region: String,
    pub temperature: i32,
    pub feels_like: i32,
    pub description: String,
    pub glyph: String,
    pub humidity: u8,
    pub wind: u16,
    pub low: i32,
    pub high: i32,
}

fn field<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for part in path {
        current = if let Ok(index) = part.parse::<usize>() {
            current.get(index)?
        } else {
            current.get(*part)?
        };
    }
    Some(current)
}

fn string(value: &Value, path: &[&str]) -> Option<String> {
    field(value, path)?.as_str().map(str::to_string)
}

fn number(value: &Value, path: &[&str]) -> Option<i32> {
    field(value, path)?
        .as_str()
        .and_then(|value| value.parse().ok())
        .or_else(|| field(value, path)?.as_i64().map(|value| value as i32))
}

fn encode_place(place: &str) -> String {
    let mut encoded = String::new();
    for byte in place.trim().bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn glyph(code: i32) -> &'static str {
    match code {
        113 => "󰖨",
        116 => "󰖕",
        119 | 122 => "󰖐",
        143 | 248 | 260 => "󰖑",
        176 | 263 | 266 | 293 | 296 | 353 => "󰼳",
        299 | 302 | 305 | 308 | 356 | 359 => "󰖖",
        179 | 227 | 230 | 323 | 326 | 329 | 332 | 335 | 338 | 368 | 371 => "󰼶",
        200 | 386 | 389 | 392 | 395 => "󰙾",
        _ => "󰖐",
    }
}

pub fn fetch(place: &str) -> Result<WeatherReport, String> {
    let url = format!("https://wttr.in/{}?format=j1", encode_place(place));
    let output = Command::new("curl")
        .args(["-fsS", "--max-time", "12", &url])
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let json: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("wttr.in returned an unknown place or invalid report: {error}"))?;
    let code =
        number(&json, &["current_condition", "0", "weatherCode"]).ok_or("weather code missing")?;
    Ok(WeatherReport {
        place: string(&json, &["nearest_area", "0", "areaName", "0", "value"]).unwrap_or_default(),
        region: string(&json, &["nearest_area", "0", "country", "0", "value"]).unwrap_or_default(),
        temperature: number(&json, &["current_condition", "0", "temp_C"]).unwrap_or(0),
        feels_like: number(&json, &["current_condition", "0", "FeelsLikeC"]).unwrap_or(0),
        description: string(
            &json,
            &["current_condition", "0", "weatherDesc", "0", "value"],
        )
        .unwrap_or_default()
        .trim()
        .to_string(),
        glyph: glyph(code).to_string(),
        humidity: number(&json, &["current_condition", "0", "humidity"])
            .unwrap_or(0)
            .clamp(0, 100) as u8,
        wind: number(&json, &["current_condition", "0", "windspeedKmph"])
            .unwrap_or(0)
            .clamp(0, u16::MAX as i32) as u16,
        low: number(&json, &["weather", "0", "mintempC"]).unwrap_or(0),
        high: number(&json, &["weather", "0", "maxtempC"]).unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::{encode_place, glyph};

    #[test]
    fn place_encoding_preserves_utf8_and_spaces() {
        assert_eq!(encode_place("Campo Grande"), "Campo%20Grande");
        assert_eq!(encode_place("São Paulo"), "S%C3%A3o%20Paulo");
    }

    #[test]
    fn weather_codes_match_the_original_icon_families() {
        assert_eq!(glyph(113), "󰖨");
        assert_eq!(glyph(302), "󰖖");
        assert_eq!(glyph(386), "󰙾");
    }
}
