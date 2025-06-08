use rstest::rstest;
use rdfless::config::{ColorConfig, string_to_color};
use colored::Color;

#[rstest]
fn test_default_color_config() {
    let config = ColorConfig::default();
    
    assert_eq!(config.subject, "blue");
    assert_eq!(config.predicate, "green");
    assert_eq!(config.object, "white");
    assert_eq!(config.literal, "red");
    assert_eq!(config.prefix, "yellow");
    assert_eq!(config.base, "yellow");
}

#[rstest]
#[case("subject", "blue", Color::Blue)]
#[case("predicate", "green", Color::Green)]
#[case("object", "white", Color::White)]
#[case("literal", "red", Color::Red)]
#[case("prefix", "yellow", Color::Yellow)]
#[case("base", "yellow", Color::Yellow)]
fn test_get_color(#[case] component: &str, #[case] color_name: &str, #[case] expected: Color) {
    let mut config = ColorConfig::default();
    
    // Override the default color for the test
    match component {
        "subject" => config.subject = color_name.to_string(),
        "predicate" => config.predicate = color_name.to_string(),
        "object" => config.object = color_name.to_string(),
        "literal" => config.literal = color_name.to_string(),
        "prefix" => config.prefix = color_name.to_string(),
        "base" => config.base = color_name.to_string(),
        _ => {}
    }
    
    assert_eq!(config.get_color(component), expected);
}

#[rstest]
#[case("black", Color::Black)]
#[case("red", Color::Red)]
#[case("green", Color::Green)]
#[case("yellow", Color::Yellow)]
#[case("blue", Color::Blue)]
#[case("magenta", Color::Magenta)]
#[case("cyan", Color::Cyan)]
#[case("white", Color::White)]
#[case("bright_black", Color::BrightBlack)]
#[case("bright_red", Color::BrightRed)]
#[case("bright_green", Color::BrightGreen)]
#[case("bright_yellow", Color::BrightYellow)]
#[case("bright_blue", Color::BrightBlue)]
#[case("bright_magenta", Color::BrightMagenta)]
#[case("bright_cyan", Color::BrightCyan)]
#[case("bright_white", Color::BrightWhite)]
#[case("unknown", Color::White)] // Default for unknown colors
fn test_string_to_color(#[case] color_name: &str, #[case] expected: Color) {
    assert_eq!(string_to_color(color_name), expected);
}