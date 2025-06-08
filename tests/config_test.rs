use colored::Color;
use rdfless::config::{string_to_color, ColorConfig, Config};
use rstest::rstest;

#[rstest]
#[case("#336699", 51, 102, 153)]
#[case("#369", 51, 102, 153)]
#[case("#FF0000", 255, 0, 0)]
#[case("#f00", 255, 0, 0)]
fn test_css_color_codes(#[case] color_code: &str, #[case] r: u8, #[case] g: u8, #[case] b: u8) {
    let color = string_to_color(color_code);
    match color {
        Color::TrueColor {
            r: red,
            g: green,
            b: blue,
        } => {
            assert_eq!(red, r);
            assert_eq!(green, g);
            assert_eq!(blue, b);
        }
        _ => panic!("Expected TrueColor, got {:?}", color),
    }
}

#[rstest]
fn test_config_serialization_deserialization() {
    // Create a default config
    let config = Config {
        colors: ColorConfig {
            subject: "blue".to_string(),
            predicate: "green".to_string(),
            object: "white".to_string(),
            literal: "red".to_string(),
            prefix: "yellow".to_string(),
            base: "yellow".to_string(),
            graph: "yellow".to_string(),
        },
        expand: false,
    };

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&config).expect("Failed to serialize config");

    // Print the YAML for debugging
    println!("Serialized YAML:\n{}", yaml);

    // Ensure the YAML doesn't contain problematic quotes
    assert!(!yaml.contains("object'"));
    assert!(!yaml.contains("literal'"));

    // Deserialize back to Config
    let deserialized_config: Config =
        serde_yaml::from_str(&yaml).expect("Failed to deserialize config");

    // Verify the deserialized config matches the original
    assert_eq!(deserialized_config.colors.subject, config.colors.subject);
    assert_eq!(
        deserialized_config.colors.predicate,
        config.colors.predicate
    );
    assert_eq!(deserialized_config.colors.object, config.colors.object);
    assert_eq!(deserialized_config.colors.literal, config.colors.literal);
    assert_eq!(deserialized_config.colors.prefix, config.colors.prefix);
    assert_eq!(deserialized_config.colors.base, config.colors.base);
    assert_eq!(deserialized_config.colors.graph, config.colors.graph);
    assert_eq!(deserialized_config.expand, config.expand);
}

#[rstest]
fn test_config_with_css_colors() {
    // Create a config with CSS color codes
    let config = Config {
        colors: ColorConfig {
            subject: "#336699".to_string(),
            predicate: "#00cc00".to_string(),
            object: "#ffffff".to_string(),
            literal: "#ff0000".to_string(),
            prefix: "#ffcc00".to_string(),
            base: "#ffcc00".to_string(),
            graph: "#ffcc00".to_string(),
        },
        expand: false,
    };

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&config).expect("Failed to serialize config");

    // Print the YAML for debugging
    println!("Serialized YAML with CSS colors:\n{}", yaml);

    // Deserialize back to Config
    let deserialized_config: Config =
        serde_yaml::from_str(&yaml).expect("Failed to deserialize config");

    // Verify the deserialized config matches the original
    assert_eq!(deserialized_config.colors.subject, config.colors.subject);
    assert_eq!(
        deserialized_config.colors.predicate,
        config.colors.predicate
    );
    assert_eq!(deserialized_config.colors.object, config.colors.object);
    assert_eq!(deserialized_config.colors.literal, config.colors.literal);
    assert_eq!(deserialized_config.colors.prefix, config.colors.prefix);
    assert_eq!(deserialized_config.colors.base, config.colors.base);
    assert_eq!(deserialized_config.colors.graph, config.colors.graph);
    assert_eq!(deserialized_config.expand, config.expand);

    // Verify that the colors are correctly parsed
    let subject_color = string_to_color(&config.colors.subject);
    match subject_color {
        Color::TrueColor { r, g, b } => {
            assert_eq!(r, 51);
            assert_eq!(g, 102);
            assert_eq!(b, 153);
        }
        _ => panic!("Expected TrueColor, got {:?}", subject_color),
    }
}
