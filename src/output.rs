//! Output formatting. Mirrors ecp: `toon` is the signal-dense default for
//! human-ish reads, `json` is the machine path. Toon encoding reuses the same
//! `etoon` crate ecp uses, so output shape is consistent across tools.

use anyhow::{Context, Result};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Toon,
    Json,
}

impl Format {
    pub fn from_json_flag(json: bool) -> Self {
        if json {
            Format::Json
        } else {
            Format::Toon
        }
    }
}

/// Encode a JSON value as toon or json.
pub fn emit(value: &Value, format: Format) -> Result<String> {
    match format {
        Format::Json => serde_json::to_string_pretty(value).context("json serialize"),
        Format::Toon => {
            let bytes = serde_json::to_vec(value).context("json serialize")?;
            _etoon::toon::encode(&bytes).map_err(|e| anyhow::anyhow!("toon encode: {e}"))
        }
    }
}
