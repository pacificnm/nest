//! Parsed configuration document.

use nest_error::{NestError, NestResult};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use tracing::debug;

use crate::codes::{
    NEST_CONFIG_PARSE_FAILED, NEST_CONFIG_SECTION_INVALID, NEST_CONFIG_SECTION_MISSING,
};

/// Parsed configuration root table.
#[derive(Debug, Clone, Default)]
pub struct ConfigDocument {
    table: toml::Table,
}

impl ConfigDocument {
    /// Creates an empty document.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Parses TOML content into a document.
    pub fn parse_toml(content: &str) -> NestResult<Self> {
        let value: toml::Value = toml::from_str(content).map_err(|error| {
            NestError::config(error.to_string())
                .with_code(NEST_CONFIG_PARSE_FAILED)
                .with_help("Check the configuration file syntax.")
        })?;
        table_from_value(value)
    }

    /// Parses JSON content into a document.
    #[cfg(feature = "json")]
    pub fn parse_json(content: &str) -> NestResult<Self> {
        let value: serde_json::Value = serde_json::from_str(content).map_err(|error| {
            NestError::config(error.to_string())
                .with_code(NEST_CONFIG_PARSE_FAILED)
                .with_help("Check the configuration file syntax.")
        })?;
        let table = json_value_to_toml(value)?;
        Ok(Self { table })
    }

    /// Returns whether a section exists.
    pub fn has_section(&self, name: &str) -> bool {
        section_value(&self.table, name).is_some()
    }

    /// Deserializes a required configuration section.
    pub fn section<T: DeserializeOwned>(&self, name: &str) -> NestResult<T> {
        debug!(config.section = name, "config section");
        let value = section_value(&self.table, name).ok_or_else(|| {
            NestError::config(format!("configuration section not found: {name}"))
                .with_code(NEST_CONFIG_SECTION_MISSING)
                .with_help(format!("Add a [{name}] section to the configuration file."))
        })?;
        deserialize_section_value(name, value)
    }

    /// Deserializes an optional configuration section.
    pub fn optional_section<T: DeserializeOwned>(&self, name: &str) -> NestResult<Option<T>> {
        if !self.has_section(name) {
            return Ok(None);
        }
        self.section(name).map(Some)
    }
}

fn table_from_value(value: toml::Value) -> NestResult<ConfigDocument> {
    let table = value.as_table().cloned().ok_or_else(|| {
        NestError::config("configuration root must be a table").with_code(NEST_CONFIG_PARSE_FAILED)
    })?;
    Ok(ConfigDocument { table })
}

fn deserialize_section_value<T: DeserializeOwned>(name: &str, value: toml::Value) -> NestResult<T> {
    let mut wrapper = toml::Table::new();
    wrapper.insert("__section__".to_string(), value);
    let content = toml::to_string(&toml::Value::Table(wrapper)).map_err(|error| {
        NestError::config(error.to_string()).with_code(NEST_CONFIG_SECTION_INVALID)
    })?;
    #[derive(Deserialize)]
    struct Section<T> {
        __section__: T,
    }
    toml::from_str::<Section<T>>(&content)
        .map_err(|error| {
            NestError::config(format!("failed to deserialize section [{name}]: {error}"))
                .with_code(NEST_CONFIG_SECTION_INVALID)
        })
        .map(|section| section.__section__)
}

fn section_value(table: &toml::Table, name: &str) -> Option<toml::Value> {
    if !name.contains('.') {
        return table.get(name).cloned();
    }

    let parts: Vec<&str> = name.split('.').collect();
    let mut current = toml::Value::Table(table.clone());
    for part in parts {
        current = current.get(part)?.clone();
    }
    Some(current)
}

#[cfg(feature = "json")]
fn json_value_to_toml(value: serde_json::Value) -> NestResult<toml::Table> {
    match value {
        serde_json::Value::Object(map) => {
            let mut table = toml::Table::new();
            for (key, value) in map {
                table.insert(key, json_value_to_toml_value(value)?);
            }
            Ok(table)
        }
        _ => Err(
            NestError::config("JSON configuration root must be an object")
                .with_code(NEST_CONFIG_PARSE_FAILED),
        ),
    }
}

#[cfg(feature = "json")]
fn json_value_to_toml_value(value: serde_json::Value) -> NestResult<toml::Value> {
    match value {
        serde_json::Value::Null => Ok(toml::Value::String(String::new())),
        serde_json::Value::Bool(value) => Ok(toml::Value::Boolean(value)),
        serde_json::Value::Number(number) => {
            if let Some(int) = number.as_i64() {
                Ok(toml::Value::Integer(int))
            } else if let Some(float) = number.as_f64() {
                Ok(toml::Value::Float(float))
            } else {
                Err(NestError::config("unsupported JSON number")
                    .with_code(NEST_CONFIG_PARSE_FAILED))
            }
        }
        serde_json::Value::String(value) => Ok(toml::Value::String(value)),
        serde_json::Value::Array(values) => values
            .into_iter()
            .map(json_value_to_toml_value)
            .collect::<NestResult<_>>()
            .map(toml::Value::Array),
        serde_json::Value::Object(map) => {
            let mut table = toml::Table::new();
            for (key, value) in map {
                table.insert(key, json_value_to_toml_value(value)?);
            }
            Ok(toml::Value::Table(table))
        }
    }
}
