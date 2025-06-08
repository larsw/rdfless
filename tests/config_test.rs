use rdfless::config::{Config, ColorConfig};
use serde_yaml;
use rstest::rstest;

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
    let deserialized_config: Config = serde_yaml::from_str(&yaml).expect("Failed to deserialize config");

    // Verify the deserialized config matches the original
    assert_eq!(deserialized_config.colors.subject, config.colors.subject);
    assert_eq!(deserialized_config.colors.predicate, config.colors.predicate);
    assert_eq!(deserialized_config.colors.object, config.colors.object);
    assert_eq!(deserialized_config.colors.literal, config.colors.literal);
    assert_eq!(deserialized_config.colors.prefix, config.colors.prefix);
    assert_eq!(deserialized_config.colors.base, config.colors.base);
    assert_eq!(deserialized_config.colors.graph, config.colors.graph);
    assert_eq!(deserialized_config.expand, config.expand);
}
