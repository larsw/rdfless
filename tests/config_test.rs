use colored::Color;
use rdfless::{string_to_color, ColorConfig, Config, OutputConfig, ThemeConfig};
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
        _ => panic!("Expected TrueColor, got {color:?}"),
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
        output: OutputConfig {
            expand: false,
            pager: false,
            auto_pager: true,
            auto_pager_threshold: 0,
        },
        theme: ThemeConfig::default(),
    };

    // Serialize to TOML
    let toml_str = toml::to_string(&config).expect("Failed to serialize config");

    // Print the TOML for debugging
    println!("Serialized TOML:\n{toml_str}");

    // Ensure the TOML doesn't contain problematic quotes
    assert!(!toml_str.contains("object'"));
    assert!(!toml_str.contains("literal'"));

    // Deserialize back to Config
    let deserialized_config: Config =
        toml::from_str(&toml_str).expect("Failed to deserialize config");

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
    assert_eq!(deserialized_config.output.expand, config.output.expand);
    assert_eq!(deserialized_config.output.pager, config.output.pager);
    assert_eq!(
        deserialized_config.output.auto_pager,
        config.output.auto_pager
    );
    assert_eq!(
        deserialized_config.output.auto_pager_threshold,
        config.output.auto_pager_threshold
    );
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
        output: OutputConfig {
            expand: false,
            pager: false,
            auto_pager: true,
            auto_pager_threshold: 0,
        },
        theme: ThemeConfig::default(),
    };

    // Serialize to TOML
    let toml_str = toml::to_string(&config).expect("Failed to serialize config");

    // Print the TOML for debugging
    println!("Serialized TOML with CSS colors:\n{toml_str}");

    // Deserialize back to Config
    let deserialized_config: Config =
        toml::from_str(&toml_str).expect("Failed to deserialize config");

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
    assert_eq!(deserialized_config.output.expand, config.output.expand);
    assert_eq!(deserialized_config.output.pager, config.output.pager);
    assert_eq!(
        deserialized_config.output.auto_pager,
        config.output.auto_pager
    );
    assert_eq!(
        deserialized_config.output.auto_pager_threshold,
        config.output.auto_pager_threshold
    );

    // Verify that the colors are correctly parsed
    let subject_color = string_to_color(&config.colors.subject);
    match subject_color {
        Color::TrueColor { r, g, b } => {
            assert_eq!(r, 51);
            assert_eq!(g, 102);
            assert_eq!(b, 153);
        }
        _ => panic!("Expected TrueColor, got {subject_color:?}"),
    }
}

#[rstest]
fn test_config_with_paging_options() {
    // Create a config with explicit paging settings
    let config = Config {
        colors: ColorConfig::default(),
        output: OutputConfig {
            expand: false,
            pager: true,
            auto_pager: false,
            auto_pager_threshold: 100,
        },
        theme: ThemeConfig::default(),
    };

    // Serialize to TOML
    let toml_str = toml::to_string(&config).expect("Failed to serialize config");
    println!("Serialized TOML with paging options:\n{toml_str}");

    // Deserialize back to Config
    let deserialized_config: Config =
        toml::from_str(&toml_str).expect("Failed to deserialize config");

    // Verify paging options are correctly serialized/deserialized
    assert_eq!(deserialized_config.output.pager, config.output.pager);
    assert_eq!(
        deserialized_config.output.auto_pager,
        config.output.auto_pager
    );
    assert_eq!(
        deserialized_config.output.auto_pager_threshold,
        config.output.auto_pager_threshold
    );
}

#[rstest]
fn test_theme_config_default() {
    let theme_config = ThemeConfig::default();

    // Test that auto-detection is enabled by default
    assert!(theme_config.auto_detect);

    // Test that we have distinct light and dark themes
    assert_ne!(
        theme_config.light_theme.predicate,
        theme_config.dark_theme.predicate
    );
    assert_ne!(
        theme_config.light_theme.object,
        theme_config.dark_theme.object
    );
}

#[rstest]
fn test_config_serialization_with_theme() {
    // Create a config with custom theme settings
    let config = Config {
        colors: ColorConfig::default(),
        output: OutputConfig::default(),
        theme: ThemeConfig {
            auto_detect: false,
            dark_theme: ColorConfig {
                subject: "cyan".to_string(),
                predicate: "magenta".to_string(),
                object: "yellow".to_string(),
                literal: "red".to_string(),
                prefix: "green".to_string(),
                base: "blue".to_string(),
                graph: "white".to_string(),
            },
            light_theme: ColorConfig {
                subject: "blue".to_string(),
                predicate: "green".to_string(),
                object: "black".to_string(),
                literal: "red".to_string(),
                prefix: "yellow".to_string(),
                base: "magenta".to_string(),
                graph: "cyan".to_string(),
            },
        },
    };

    // Serialize to TOML
    let toml_str = toml::to_string(&config).expect("Failed to serialize config");
    println!("Serialized TOML with custom theme:\n{toml_str}");

    // Deserialize back to Config
    let deserialized_config: Config =
        toml::from_str(&toml_str).expect("Failed to deserialize config");

    // Verify theme settings are correctly serialized/deserialized
    assert_eq!(
        deserialized_config.theme.auto_detect,
        config.theme.auto_detect
    );
    assert_eq!(
        deserialized_config.theme.dark_theme.subject,
        config.theme.dark_theme.subject
    );
    assert_eq!(
        deserialized_config.theme.light_theme.predicate,
        config.theme.light_theme.predicate
    );
}

#[rstest]
fn test_default_auto_pager_enabled() {
    let config = Config::default();

    // Test that auto_pager is enabled by default
    assert!(config.output.auto_pager);

    // Test that auto_pager_threshold defaults to 0 (use terminal height)
    assert_eq!(config.output.auto_pager_threshold, 0);
}
