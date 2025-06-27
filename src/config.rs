// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use anyhow::{Context, Result};
use colored::Color;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use toml;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub colors: ColorConfig,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeConfig {
    #[serde(default)]
    pub auto_detect: bool,
    #[serde(default)]
    pub dark_theme: ColorConfig,
    #[serde(default)]
    pub light_theme: ColorConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutputConfig {
    #[serde(default)]
    pub expand: bool,
    #[serde(default)]
    pub pager: bool,
    /// Automatically enable paging when output is longer than screen height
    #[serde(default = "default_auto_pager")]
    pub auto_pager: bool,
    /// Threshold for auto-paging (in lines). If 0, uses terminal height
    #[serde(default)]
    pub auto_pager_threshold: usize,
}

fn default_auto_pager() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColorConfig {
    pub subject: String,
    pub predicate: String,
    #[serde(rename = "object")]
    pub object: String,
    #[serde(rename = "literal")]
    pub literal: String,
    pub prefix: String,
    pub base: String,
    pub graph: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        ThemeConfig {
            auto_detect: true,
            dark_theme: ColorConfig {
                subject: "blue".to_string(),
                predicate: "green".to_string(),
                object: "white".to_string(),
                literal: "red".to_string(),
                prefix: "yellow".to_string(),
                base: "yellow".to_string(),
                graph: "yellow".to_string(),
            },
            light_theme: ColorConfig {
                subject: "blue".to_string(),
                predicate: "#006400".to_string(), // dark green
                object: "black".to_string(),
                literal: "#8B0000".to_string(), // dark red
                prefix: "#B8860B".to_string(),  // dark goldenrod
                base: "#B8860B".to_string(),    // dark goldenrod
                graph: "#B8860B".to_string(),   // dark goldenrod
            },
        }
    }
}

impl Default for ColorConfig {
    fn default() -> Self {
        ColorConfig {
            subject: "blue".to_string(),
            predicate: "green".to_string(),
            object: "white".to_string(),
            literal: "red".to_string(),
            prefix: "yellow".to_string(),
            base: "yellow".to_string(),
            graph: "yellow".to_string(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        OutputConfig {
            expand: false,
            pager: false,
            auto_pager: default_auto_pager(),
            auto_pager_threshold: 0,
        }
    }
}

impl ColorConfig {
    pub fn get_color(&self, name: &str) -> Color {
        match name {
            "subject" => string_to_color(&self.subject),
            "predicate" => string_to_color(&self.predicate),
            "object" => string_to_color(&self.object),
            "literal" => string_to_color(&self.literal),
            "prefix" => string_to_color(&self.prefix),
            "base" => string_to_color(&self.base),
            "graph" => string_to_color(&self.graph),
            _ => Color::White,
        }
    }
}

pub fn string_to_color(color_name: &str) -> Color {
    // Check if the color is a CSS hex color code (e.g., #336699)
    if color_name.starts_with('#') && (color_name.len() == 7 || color_name.len() == 4) {
        return parse_hex_color(color_name).unwrap_or(Color::White);
    }

    // Otherwise, try to match named colors
    match color_name.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "bright_black" => Color::BrightBlack,
        "bright_red" => Color::BrightRed,
        "bright_green" => Color::BrightGreen,
        "bright_yellow" => Color::BrightYellow,
        "bright_blue" => Color::BrightBlue,
        "bright_magenta" => Color::BrightMagenta,
        "bright_cyan" => Color::BrightCyan,
        "bright_white" => Color::BrightWhite,
        _ => Color::White, // Default to white for unknown colors
    }
}

// Parse a CSS hex color code to an RGB Color
fn parse_hex_color(hex: &str) -> Option<Color> {
    // Remove the leading '#'
    let hex = hex.trim_start_matches('#');

    // Handle both 3-digit and 6-digit hex codes
    let (r, g, b) = if hex.len() == 6 {
        // 6-digit hex code: #RRGGBB
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        (r, g, b)
    } else if hex.len() == 3 {
        // 3-digit hex code: #RGB (equivalent to #RRGGBB)
        let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
        (r, g, b)
    } else {
        return None;
    };

    Some(Color::TrueColor { r, g, b })
}

pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        create_default_config(&config_path)?;
    }

    // Try to read and parse the config file
    let config_str = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    match toml::from_str::<Config>(&config_str) {
        Ok(config) => Ok(config),
        Err(_e) => {
            // If parsing fails, delete the old config file and create a new one
            eprintln!("Warning: Failed to parse existing config file. Creating a new one.");
            fs::remove_file(&config_path).with_context(|| {
                format!(
                    "Failed to remove old config file: {}",
                    config_path.display()
                )
            })?;

            create_default_config(&config_path)?;

            // Read and parse the new config file
            let new_config_str = fs::read_to_string(&config_path).with_context(|| {
                format!("Failed to read new config file: {}", config_path.display())
            })?;

            let config: Config = toml::from_str(&new_config_str)
                .with_context(|| "Failed to parse new config file")?;

            Ok(config)
        }
    }
}

fn get_config_path() -> Result<PathBuf> {
    let home = home_dir().context("Could not find home directory")?;
    let config_dir = home.join(".local").join("rdfless");
    let config_path = config_dir.join("config.toml");

    Ok(config_path)
}

fn create_default_config(config_path: &PathBuf) -> Result<()> {
    let config_dir = config_path.parent().unwrap();

    if !config_dir.exists() {
        fs::create_dir_all(config_dir).with_context(|| {
            format!(
                "Failed to create config directory: {}",
                config_dir.display()
            )
        })?;
    }

    let default_config = Config::default();
    let toml_str =
        toml::to_string_pretty(&default_config).context("Failed to serialize default config")?;

    fs::write(config_path, toml_str).with_context(|| {
        format!(
            "Failed to write default config to: {}",
            config_path.display()
        )
    })?;

    Ok(())
}

// Function to detect if terminal has a light background
pub fn is_light_background() -> bool {
    if let Ok(theme) = termbg::theme(std::time::Duration::from_millis(100)) {
        matches!(theme, termbg::Theme::Light)
    } else {
        // Default to dark if detection fails
        false
    }
}

// Function to get the appropriate color config based on background detection
pub fn get_effective_colors(config: &Config) -> ColorConfig {
    if config.theme.auto_detect {
        if is_light_background() {
            config.theme.light_theme.clone()
        } else {
            config.theme.dark_theme.clone()
        }
    } else {
        // Use the explicitly configured colors
        config.colors.clone()
    }
}
