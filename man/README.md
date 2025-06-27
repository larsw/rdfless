# Manual Pages for rdfless

This directory contains manual pages for `rdfless` and its configuration format.

## Files

- `rdfless.1` - Manual page for the rdfless command (section 1)
- `rdfless-config.5` - Manual page for the configuration file format (section 5)

## Installation

### Using the Makefile

To install the manual pages system-wide (requires sudo):

```bash
sudo make install-man
```

To uninstall:

```bash
sudo make uninstall-man
```

### Manual Installation

To install manually:

```bash
# Copy manual pages to system directories
sudo cp man/rdfless.1 /usr/local/share/man/man1/
sudo cp man/rdfless-config.5 /usr/local/share/man/man5/

# Set proper permissions
sudo chmod 644 /usr/local/share/man/man1/rdfless.1
sudo chmod 644 /usr/local/share/man/man5/rdfless-config.5
```

### User-specific Installation

For user-specific installation (no sudo required):

```bash
# Create user manual directories
mkdir -p ~/.local/share/man/man1 ~/.local/share/man/man5

# Copy manual pages
cp man/rdfless.1 ~/.local/share/man/man1/
cp man/rdfless-config.5 ~/.local/share/man/man5/

# Add to MANPATH if needed
echo 'export MANPATH="$HOME/.local/share/man:$MANPATH"' >> ~/.bashrc
source ~/.bashrc
```

## Viewing Manual Pages

After installation, you can view the manual pages with:

```bash
# View the main rdfless command manual
man rdfless

# View the configuration file manual
man 5 rdfless-config
# or
man rdfless-config
```

## Testing Manual Pages

To preview the manual pages without installation:

```bash
# Preview the command manual
man ./man/rdfless.1

# Preview the configuration manual
man ./man/rdfless-config.5
```

## Format

The manual pages are written in `troff` format with `man` macros, following the standard Unix manual page conventions. They include:

- Standard sections (NAME, SYNOPSIS, DESCRIPTION, OPTIONS, etc.)
- Cross-references between pages
- Examples and usage patterns
- Proper formatting for different terminal widths

## Maintenance

When updating the manual pages:

1. Edit the `.1` or `.5` files directly
2. Test with `man ./man/filename` to verify formatting
3. Update version numbers and dates as needed
4. Ensure consistency between command-line help and manual content
