// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
// 
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use colored::Color;
use serde::{Deserialize, Serialize};
use dirs::home_dir;
use serde_yaml2 as serde_yaml;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub colors: ColorConfig,
    #[serde(default)]
    pub expand: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            colors: ColorConfig::default(),
            expand: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColorConfig {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub literal: String,
    pub prefix: String,
    pub base: String,
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
            _ => Color::White,
        }
    }
}

pub fn string_to_color(color_name: &str) -> Color {
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

pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        create_default_config(&config_path)?;
    }

    let config_str = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    let config: Config = serde_yaml::from_str(&config_str)
        .with_context(|| "Failed to parse config file")?;

    Ok(config)
}


fn get_config_path() -> Result<PathBuf> {
    let home = home_dir().context("Could not find home directory")?;
    let config_dir = home.join(".local").join("rdfless");
    let config_path = config_dir.join("config.yml");

    Ok(config_path)
}


fn create_default_config(config_path: &PathBuf) -> Result<()> {
    let config_dir = config_path.parent().unwrap();

    if !config_dir.exists() {
        fs::create_dir_all(config_dir)
            .with_context(|| format!("Failed to create config directory: {}", config_dir.display()))?;
    }

    let default_config = Config::default();
    let yaml = serde_yaml::to_string(&default_config)
        .context("Failed to serialize default config")?;

    fs::write(config_path, yaml)
        .with_context(|| format!("Failed to write default config to: {}", config_path.display()))?;

    Ok(())
}
