---
# sway-configurator

A standalone GUI editor for Sway configuration — manage a live session safely with generated config + live apply.

## Description

sway-configurator is a Rust + GTK4/libadwaita application for configuring Sway and Waybar. It keeps settings in `~/.config/sway-configurator/settings.toml`, applies changes from one global **Apply** action, and updates generated runtime files for Sway/Waybar/theme integration.

## Features

- **Single Apply / Revert flow** across all pages (including Themes)
- **Unsaved changes tracking** with immediate Apply bar visibility on first edit
- **Themes + overrides** (wallpaper, font family/size, gaps, border width, waybar opacity)
- **Built-in + user theme discovery** from `~/.config/sway/themes` and `~/source/SwayConfig/themes`
- **Waybar integration** with safe reload behavior
- **Settings persisted** to `$XDG_CONFIG_HOME/sway-configurator/settings.toml`
- **Headless-safe test suite** with `--no-default-features`

## Settings Pages

The app exposes eight sidebar pages:

1. **Displays** — monitor layout, resolution, position, scale, transform (rotation/flip)
2. **Inputs** — keyboard layout/variant/options/repeat rate, touchpad tap/scroll/DWT/acceleration
3. **Idle / Lock** — swayidle timeout, screen lock command, screen-off timeout
4. **Waybar** — enable/disable, position, modules configuration
5. **Autostart** — exec entries with description and enable/disable toggle
6. **Notifications** — notification daemon settings
7. **Themes** — select palette + optional override rows (checkbox enables override)
8. **General** — environment defaults (e.g. terminal command + args)

## Building from Source

> **Note:** Building requires the full GTK4 development stack. On a minimal Sway setup you are likely missing most of these. Install all system dependencies before running `cargo build`.

### System Dependencies

This app builds against GTK4 and libadwaita, which pull in a chain of system libraries that must be present via `pkg-config`. Install them all at once:

**Debian / Ubuntu:**
```bash
sudo apt install \
  libgtk-4-dev \
  libadwaita-1-dev \
  libpango1.0-dev \
  libcairo2-dev \
  libgdk-pixbuf-2.0-dev \
  libgraphene-1.0-dev \
  pkg-config \
  build-essential
```

**Arch Linux:**
```bash
sudo pacman -S gtk4 libadwaita pango cairo gdk-pixbuf2 graphene pkgconf base-devel
```

**Fedora:**
```bash
sudo dnf install gtk4-devel libadwaita-devel pango-devel cairo-devel gdk-pixbuf2-devel graphene-devel pkg-config
```

### Rust Toolchain

Requires Rust stable 1.70 or newer. Install via [rustup](https://rustup.rs):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build and Run

```bash
cargo build --release
cargo run
```

### Build Without GTK (headless / library mode)

The core model and config layers have no GTK dependency and can be built standalone:
```bash
cargo build --no-default-features
```

### Tests

Tests run without GTK — no system libraries required for the test suite:
```bash
cargo test --no-default-features
```

## Packaging (WIP)

Initial Debian packaging metadata is included via `cargo-deb`.

```bash
cargo install cargo-deb
cargo deb
```

Generated package artifacts are placed in `target/debian/`.

## Architecture

```
src/
├── model/              # Pure data model (no GTK) — serde-serializable Settings structs
├── config/             # Persistence, IPC, detection, rendering, apply pipeline
│   ├── store.rs        # TOML load/save with XDG path resolution
│   ├── render_model.rs # Settings → intermediate render representation
│   ├── apply.rs        # Write config fragments, optionally reload sway/waybar
│   ├── detect.rs       # Parse live swaymsg JSON (outputs, inputs)
│   ├── feature.rs      # Runtime feature detection (which, /sys)
│   └── swaymsg.rs      # Sway IPC wrapper
└── ui/                 # GTK4/libadwaita GUI (feature-gated on `gtk` feature)
    ├── window.rs       # Main window — NavigationSplitView + GtkStack
    ├── apply_bar.rs    # Apply / Revert bar
    └── pages/          # One file per settings page
```

## Managed Config Files

The app writes these managed paths. Keep your main Sway config including generated fragments:

| File | Purpose |
|------|---------|
| `~/.config/sway/conf.d/outputs.conf` | Display layout |
| `~/.config/sway/conf.d/inputs.conf` | Keyboard & touchpad |
| `~/.config/sway/conf.d/idle.conf` | Idle/lock config |
| `~/.config/sway/conf.d/autostart.conf` | Autostart entries |
| `~/.config/waybar/config.jsonc` | Waybar bar config |
| `~/.config/waybar/style.css` | Waybar style (theme variables) |
| `~/.config/gtk-3.0/settings.ini` | GTK settings from selected theme |
| `~/.config/gtk-4.0/settings.ini` | GTK settings from selected theme |
| `~/.config/sway-theme` | Pointer to active theme `.env` file |

Make sure your main sway config includes `conf.d`:
```
include ~/.config/sway/conf.d/*
```

## Settings File

`$XDG_CONFIG_HOME/sway-configurator/settings.toml`  
Defaults to `~/.config/sway-configurator/settings.toml`

## Screenshots

*(Add screenshots here)*

## License

MIT — © 2026 Daniel Vincent

---