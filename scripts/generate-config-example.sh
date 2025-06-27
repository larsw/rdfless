#!/bin/bash
# Generate example configuration file for rdfless

cat > rdfless-config-example.toml << 'EOF'
# Example configuration file for rdfless
# Place this file at ~/.config/rdfless/config.toml (Linux/macOS)
# or %APPDATA%\rdfless\config.toml (Windows)

# Default color configuration
[colors]
subject = "blue"
predicate = "green"
object = "white"
literal = "red"
prefix = "yellow"
base = "yellow"
graph = "yellow"

# Output formatting settings
[output]
# Whether to expand prefixes by default
expand = false

# Whether to use pager by default
pager = false

# Automatically enable paging for large outputs
auto_pager = true

# Threshold for auto-paging (0 = use terminal height)
auto_pager_threshold = 0

# Theme configuration
[theme]
# Automatically detect terminal background
auto_detect = true

# Colors for dark terminal backgrounds
[theme.dark_theme]
subject = "blue"
predicate = "green"
object = "white"
literal = "red"
prefix = "yellow"
base = "yellow"
graph = "yellow"

# Colors for light terminal backgrounds
[theme.light_theme]
subject = "blue"
predicate = "#006400"    # dark green
object = "black"
literal = "#8B0000"     # dark red
prefix = "#B8860B"      # dark goldenrod
base = "#B8860B"        # dark goldenrod
graph = "#B8860B"       # dark goldenrod
EOF

echo "Example configuration file generated: rdfless-config-example.toml"
echo "Copy this to ~/.config/rdfless/config.toml to use it"
