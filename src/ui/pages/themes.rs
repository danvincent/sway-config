use crate::model::theme::{ThemeOverrides, ThemeSelection};
use crate::state::AppState;
/// Themes configuration page
use gtk4::prelude::*;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

/// A parsed .env theme with colors and metadata
#[derive(Debug, Clone)]
pub struct ThemeEnv {
    pub name: String,
    pub source: String,
    pub path: String,
    pub vars: HashMap<String, String>,
}

impl ThemeEnv {
    /// Key colors to show in the palette swatch (ordered for visual effect)
    const SWATCH_KEYS: &'static [&'static str] = &[
        "COLOR_BASE",
        "COLOR_SURFACE0",
        "COLOR_SURFACE1",
        "COLOR_OVERLAY0",
        "COLOR_TEXT",
        "COLOR_LAVENDER",
        "COLOR_BLUE",
        "COLOR_SAPPHIRE",
        "COLOR_TEAL",
        "COLOR_GREEN",
        "COLOR_YELLOW",
        "COLOR_PEACH",
        "COLOR_RED",
        "COLOR_MAUVE",
        "COLOR_PINK",
        "COLOR_ROSEWATER",
    ];

    fn swatch_colors(&self) -> Vec<(f64, f64, f64)> {
        Self::SWATCH_KEYS
            .iter()
            .filter_map(|k| self.vars.get(*k).and_then(|hex| parse_hex_color(hex)))
            .collect()
    }
}

fn parse_hex_color(hex: &str) -> Option<(f64, f64, f64)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
    Some((r, g, b))
}

/// Scan directories for .env theme files and parse them
pub fn scan_env_themes(custom_path: Option<&str>) -> Vec<ThemeEnv> {
    let mut themes = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let mut dirs: Vec<(PathBuf, String)> = Vec::new();
    // User override path
    if let Some(p) = custom_path {
        dirs.push((PathBuf::from(p), "user".to_string()));
    }
    // Default user themes path
    if let Ok(home) = std::env::var("HOME") {
        dirs.push((
            PathBuf::from(format!("{}/.config/sway/themes", home)),
            "user".to_string(),
        ));
        dirs.push((
            PathBuf::from(format!("{}/source/SwayConfig/themes", home)),
            "built-in".to_string(),
        ));
    }

    for (dir, source) in dirs {
        if !dir.exists() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut dir_themes: Vec<ThemeEnv> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "env").unwrap_or(false))
            .filter_map(|e| {
                let path = e.path();
                let content = std::fs::read_to_string(&path).ok()?;
                let vars = crate::config::apply::parse_env_file(&content);
                let name = path.file_stem()?.to_str()?.to_string();
                if seen.contains(&name) {
                    return None;
                }
                Some(ThemeEnv {
                    name: name.clone(),
                    source: source.clone(),
                    path: path.to_string_lossy().into(),
                    vars,
                })
            })
            .collect();
        dir_themes.sort_by(|a, b| a.name.cmp(&b.name));
        for t in dir_themes {
            seen.insert(t.name.clone());
            themes.push(t);
        }
    }
    themes
}

/// Read the active theme path from ~/.config/sway-theme
pub fn active_theme_path() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let content = std::fs::read_to_string(format!("{}/.config/sway-theme", home)).ok()?;
    let path = content.trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

/// Themes page
#[allow(deprecated)]
pub struct ThemesPage {
    widget: gtk4::Box,
    list_box: gtk4::ListBox,
    wallpaper_check: gtk4::CheckButton,
    wallpaper_row: libadwaita::EntryRow,
    font_family_check: gtk4::CheckButton,
    font_family_button: gtk4::FontButton,
    font_size_check: gtk4::CheckButton,
    font_size_row: libadwaita::SpinRow,
    gap_inner_check: gtk4::CheckButton,
    gap_inner_row: libadwaita::SpinRow,
    gap_outer_check: gtk4::CheckButton,
    gap_outer_row: libadwaita::SpinRow,
    border_width_check: gtk4::CheckButton,
    border_width_row: libadwaita::SpinRow,
    waybar_opacity_check: gtk4::CheckButton,
    waybar_opacity_row: libadwaita::SpinRow,
    themes: Rc<RefCell<Vec<ThemeEnv>>>,
    loading: Rc<std::cell::Cell<bool>>,
    /// Suppresses model updates while `load_overrides_into_ui` restores state
    loading_overrides: Rc<std::cell::Cell<bool>>,
    app_state: Rc<RefCell<AppState>>,
}

/// Build a wrapper box containing [CheckButton | SpinRow].
/// The check controls `row.set_sensitive` — the row starts insensitive.
/// Returns (wrapper, row, check).
fn make_checkable_spin(
    title: &str,
    min: f64,
    max: f64,
    step: f64,
    digits: u32,
) -> (gtk4::Box, libadwaita::SpinRow, gtk4::CheckButton) {
    let row = libadwaita::SpinRow::with_range(min, max, step);
    row.set_title(title);
    row.set_digits(digits);
    row.set_hexpand(true);
    row.set_sensitive(false);

    let check = gtk4::CheckButton::new();
    check.set_valign(gtk4::Align::Center);
    check.set_margin_start(8);
    check.set_margin_end(4);
    {
        let r = row.clone();
        check.connect_active_notify(move |c| r.set_sensitive(c.is_active()));
    }

    let wrapper = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    wrapper.append(&check);
    wrapper.append(&row);
    (wrapper, row, check)
}

/// Build a wrapper box containing [CheckButton | EntryRow].
/// The check controls `row.set_sensitive` — the row starts insensitive.
/// Returns (wrapper, row, check).
fn make_checkable_entry(title: &str) -> (gtk4::Box, libadwaita::EntryRow, gtk4::CheckButton) {
    let row = libadwaita::EntryRow::new();
    row.set_title(title);
    row.set_show_apply_button(true);
    row.set_hexpand(true);
    row.set_sensitive(false);

    let check = gtk4::CheckButton::new();
    check.set_valign(gtk4::Align::Center);
    check.set_margin_start(8);
    check.set_margin_end(4);
    {
        let r = row.clone();
        check.connect_active_notify(move |c| r.set_sensitive(c.is_active()));
    }

    let wrapper = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    wrapper.append(&check);
    wrapper.append(&row);
    (wrapper, row, check)
}

/// Build a wrapper box containing [CheckButton | Label + Font picker].
/// The check controls picker sensitivity — picker starts insensitive.
/// Returns (wrapper, picker_button, check).
#[allow(deprecated)]
fn make_checkable_font(title: &str) -> (gtk4::Box, gtk4::FontButton, gtk4::CheckButton) {
    let check = gtk4::CheckButton::new();
    check.set_valign(gtk4::Align::Center);
    check.set_margin_start(8);
    check.set_margin_end(4);

    let label = gtk4::Label::new(Some(title));
    label.set_halign(gtk4::Align::Start);
    label.add_css_class("caption");

    let button = gtk4::FontButton::new();
    button.set_hexpand(true);
    button.set_use_font(true);
    button.set_use_size(true);
    button.set_sensitive(false);

    {
        let b = button.clone();
        check.connect_active_notify(move |c| b.set_sensitive(c.is_active()));
    }

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    content.append(&label);
    content.append(&button);

    let wrapper = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    wrapper.append(&check);
    wrapper.append(&content);
    (wrapper, button, check)
}

#[allow(deprecated)]
fn selected_font_family(button: &gtk4::FontButton) -> Option<String> {
    let font = button.font()?;
    let desc = gtk4::pango::FontDescription::from_string(font.as_str());
    desc.family()
        .map(|f| f.to_string())
        .filter(|f| !f.trim().is_empty())
}

impl ThemesPage {
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);

        let clamp = libadwaita::Clamp::new();
        clamp.set_maximum_size(800);

        let outer_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        outer_box.set_margin_top(12);
        outer_box.set_margin_bottom(12);
        outer_box.set_margin_start(12);
        outer_box.set_margin_end(12);

        // ── Theme list ────────────────────────────────────────────────────────
        let list_group = libadwaita::PreferencesGroup::new();
        list_group.set_title("Theme");
        list_group.set_description(Some(
            "Select a colour palette — applies sway colours, waybar style, and GTK theme",
        ));

        let list_box = gtk4::ListBox::new();
        list_box.set_css_classes(&["boxed-list"]);
        list_box.set_selection_mode(gtk4::SelectionMode::Single);
        list_group.add(&list_box);
        outer_box.append(&list_group);

        // ── Appearance overrides ─────────────────────────────────────────────
        let appearance_group = libadwaita::PreferencesGroup::new();
        appearance_group.set_title("Appearance Overrides");
        appearance_group
            .set_description(Some("Check a row to override the theme's built-in value"));

        let (wallpaper_wrapper, wallpaper_row, wallpaper_check) =
            make_checkable_entry("Wallpaper path");
        let (font_family_wrapper, font_family_button, font_family_check) =
            make_checkable_font("Font family");
        let (font_size_wrapper, font_size_row, font_size_check) =
            make_checkable_spin("Font size", 6.0, 72.0, 1.0, 0);
        let (gap_inner_wrapper, gap_inner_row, gap_inner_check) =
            make_checkable_spin("Inner gap (px)", 0.0, 100.0, 1.0, 0);
        let (gap_outer_wrapper, gap_outer_row, gap_outer_check) =
            make_checkable_spin("Outer gap (px)", 0.0, 100.0, 1.0, 0);
        let (border_width_wrapper, border_width_row, border_width_check) =
            make_checkable_spin("Border width (px)", 0.0, 20.0, 1.0, 0);
        let (waybar_opacity_wrapper, waybar_opacity_row, waybar_opacity_check) =
            make_checkable_spin("Waybar opacity", 0.0, 1.0, 0.05, 2);

        appearance_group.add(&wallpaper_wrapper);
        appearance_group.add(&font_family_wrapper);
        appearance_group.add(&font_size_wrapper);
        appearance_group.add(&gap_inner_wrapper);
        appearance_group.add(&gap_outer_wrapper);
        appearance_group.add(&border_width_wrapper);
        appearance_group.add(&waybar_opacity_wrapper);
        outer_box.append(&appearance_group);

        clamp.set_child(Some(&outer_box));
        scrolled.set_child(Some(&clamp));
        widget.append(&scrolled);

        let themes: Rc<RefCell<Vec<ThemeEnv>>> = Rc::new(RefCell::new(Vec::new()));
        let loading = Rc::new(std::cell::Cell::new(false));
        let loading_overrides = Rc::new(std::cell::Cell::new(false));

        // ── Theme selection signal ────────────────────────────────────────────
        {
            let state = Rc::clone(&app_state);
            let themes_ref = Rc::clone(&themes);
            let loading_ref = Rc::clone(&loading);
            list_box.connect_row_selected(move |_, row| {
                if loading_ref.get() {
                    return;
                }
                let Some(row) = row else { return };
                let idx = row.index() as usize;
                let t = themes_ref.borrow();
                let Some(theme) = t.get(idx) else { return };
                let sel = ThemeSelection::with_path(&theme.name, &theme.source, &theme.path);
                state.borrow_mut().settings_mut().theme = Some(sel);
                state.borrow_mut().mark_dirty();
            });
        }

        // ── Wallpaper signals ─────────────────────────────────────────────────
        {
            let state = Rc::clone(&app_state);
            let row_ref = wallpaper_row.clone();
            let check_ref = wallpaper_check.clone();
            let lo = Rc::clone(&loading_overrides);
            wallpaper_row.connect_apply(move |_| {
                if lo.get() {
                    return;
                }
                let v = row_ref.text().to_string();
                let v = v.trim().to_string();
                state.borrow_mut().settings_mut().theme_overrides.wallpaper =
                    if check_ref.is_active() && !v.is_empty() {
                        Some(v)
                    } else {
                        None
                    };
                state.borrow_mut().mark_dirty();
            });
        }
        {
            let state = Rc::clone(&app_state);
            let row_ref = wallpaper_row.clone();
            let lo = Rc::clone(&loading_overrides);
            wallpaper_check.connect_active_notify(move |c| {
                if lo.get() {
                    return;
                }
                if !c.is_active() {
                    state.borrow_mut().settings_mut().theme_overrides.wallpaper = None;
                    state.borrow_mut().mark_dirty();
                } else {
                    let v = row_ref.text().to_string();
                    let v = v.trim().to_string();
                    if !v.is_empty() {
                        state.borrow_mut().settings_mut().theme_overrides.wallpaper = Some(v);
                        state.borrow_mut().mark_dirty();
                    }
                }
            });
        }

        // ── Font family signals ───────────────────────────────────────────────
        {
            let state = Rc::clone(&app_state);
            let btn_ref = font_family_button.clone();
            let lo = Rc::clone(&loading_overrides);
            font_family_check.connect_active_notify(move |c| {
                if lo.get() {
                    return;
                }
                if !c.is_active() {
                    state
                        .borrow_mut()
                        .settings_mut()
                        .theme_overrides
                        .font_family = None;
                    state.borrow_mut().mark_dirty();
                } else {
                    if let Some(v) = selected_font_family(&btn_ref) {
                        state
                            .borrow_mut()
                            .settings_mut()
                            .theme_overrides
                            .font_family = Some(v);
                        state.borrow_mut().mark_dirty();
                    }
                }
            });
        }
        {
            let state = Rc::clone(&app_state);
            let check_ref = font_family_check.clone();
            let lo = Rc::clone(&loading_overrides);
            #[allow(deprecated)]
            font_family_button.connect_font_set(move |btn| {
                if lo.get() {
                    return;
                }
                if !check_ref.is_active() {
                    return;
                }
                if let Some(v) = selected_font_family(btn) {
                    state
                        .borrow_mut()
                        .settings_mut()
                        .theme_overrides
                        .font_family = Some(v);
                    state.borrow_mut().mark_dirty();
                }
            });
        }

        // ── Spin row check + value signals ────────────────────────────────────
        macro_rules! spin_signals {
            ($row:expr, $check:expr, $field:ident, $ty:ty) => {{
                // Check toggled → enable/disable override
                {
                    let state = Rc::clone(&app_state);
                    let row_ref = $row.clone();
                    let lo = Rc::clone(&loading_overrides);
                    $check.connect_active_notify(move |c| {
                        if lo.get() {
                            return;
                        }
                        state.borrow_mut().settings_mut().theme_overrides.$field = if c.is_active()
                        {
                            Some(row_ref.value() as $ty)
                        } else {
                            None
                        };
                        state.borrow_mut().mark_dirty();
                    });
                }
                // Value changed → update only when checked
                {
                    let state = Rc::clone(&app_state);
                    let check_ref = $check.clone();
                    let row_ref = $row.clone();
                    let lo = Rc::clone(&loading_overrides);
                    $row.connect_value_notify(move |_| {
                        if lo.get() {
                            return;
                        }
                        if check_ref.is_active() {
                            state.borrow_mut().settings_mut().theme_overrides.$field =
                                Some(row_ref.value() as $ty);
                            state.borrow_mut().mark_dirty();
                        }
                    });
                }
            }};
        }
        spin_signals!(font_size_row, font_size_check, font_size, u32);
        spin_signals!(gap_inner_row, gap_inner_check, gap_inner, u32);
        spin_signals!(gap_outer_row, gap_outer_check, gap_outer, u32);
        spin_signals!(border_width_row, border_width_check, border_width, u32);
        spin_signals!(
            waybar_opacity_row,
            waybar_opacity_check,
            waybar_opacity,
            f64
        );

        ThemesPage {
            widget,
            list_box,
            wallpaper_check,
            wallpaper_row,
            font_family_check,
            font_family_button,
            font_size_check,
            font_size_row,
            gap_inner_check,
            gap_inner_row,
            gap_outer_check,
            gap_outer_row,
            border_width_check,
            border_width_row,
            waybar_opacity_check,
            waybar_opacity_row,
            themes,
            loading,
            loading_overrides,
            app_state,
        }
    }

    pub fn on_navigate(&self) {
        let state = self.app_state.borrow();
        let theme = state.settings().theme.clone();
        let custom_path = state.settings().custom_themes_path.clone();
        let overrides = state.settings().theme_overrides.clone();
        drop(state);

        // If the stored theme has no path (saved before the path field existed),
        // resolve it by scanning now and update settings silently (no dirty mark).
        let selected_path = match theme {
            Some(ref sel) if sel.path.is_empty() => {
                let themes = scan_env_themes(custom_path.as_deref());
                if let Some(found) = themes.iter().find(|t| t.name == sel.name) {
                    let mut s = self.app_state.borrow_mut();
                    if let Some(ref mut t) = s.settings_mut().theme {
                        t.path = found.path.clone();
                    }
                    Some(found.path.clone())
                } else {
                    None
                }
            }
            Some(ref sel) => Some(sel.path.clone()),
            None => None,
        };

        self.load_overrides_into_ui(&overrides);
        self.load_themes(selected_path.as_deref(), custom_path.as_deref());
    }

    /// Persist current controls into `theme_overrides`.
    /// Called by the global Apply action so page-local widgets are always honored.
    pub fn sync_overrides_into_state(&self) {
        let mut state = self.app_state.borrow_mut();
        let overrides = &mut state.settings_mut().theme_overrides;

        let wp = self.wallpaper_row.text().trim().to_string();
        overrides.wallpaper = if self.wallpaper_check.is_active() && !wp.is_empty() {
            Some(wp)
        } else {
            None
        };

        overrides.font_family = if self.font_family_check.is_active() {
            selected_font_family(&self.font_family_button)
        } else {
            None
        };

        overrides.font_size = if self.font_size_check.is_active() {
            Some(self.font_size_row.value() as u32)
        } else {
            None
        };

        overrides.gap_inner = if self.gap_inner_check.is_active() {
            Some(self.gap_inner_row.value() as u32)
        } else {
            None
        };

        overrides.gap_outer = if self.gap_outer_check.is_active() {
            Some(self.gap_outer_row.value() as u32)
        } else {
            None
        };

        overrides.border_width = if self.border_width_check.is_active() {
            Some(self.border_width_row.value() as u32)
        } else {
            None
        };

        overrides.waybar_opacity = if self.waybar_opacity_check.is_active() {
            Some(self.waybar_opacity_row.value())
        } else {
            None
        };
    }

    fn load_overrides_into_ui(&self, overrides: &ThemeOverrides) {
        self.loading_overrides.set(true);

        // Wallpaper
        self.wallpaper_check
            .set_active(overrides.wallpaper.is_some());
        if let Some(ref wp) = overrides.wallpaper {
            self.wallpaper_row.set_text(wp);
        }

        // Font family
        self.font_family_check
            .set_active(overrides.font_family.is_some());
        if let Some(ref ff) = overrides.font_family {
            #[allow(deprecated)]
            self.font_family_button.set_font(&format!("{} 10", ff));
        }

        // Spin rows: set check then value
        self.font_size_check
            .set_active(overrides.font_size.is_some());
        self.font_size_row
            .set_value(overrides.font_size.unwrap_or(10) as f64);

        self.gap_inner_check
            .set_active(overrides.gap_inner.is_some());
        self.gap_inner_row
            .set_value(overrides.gap_inner.unwrap_or(0) as f64);

        self.gap_outer_check
            .set_active(overrides.gap_outer.is_some());
        self.gap_outer_row
            .set_value(overrides.gap_outer.unwrap_or(0) as f64);

        self.border_width_check
            .set_active(overrides.border_width.is_some());
        self.border_width_row
            .set_value(overrides.border_width.unwrap_or(2) as f64);

        self.waybar_opacity_check
            .set_active(overrides.waybar_opacity.is_some());
        self.waybar_opacity_row
            .set_value(overrides.waybar_opacity.unwrap_or(1.0));

        self.loading_overrides.set(false);
    }

    pub fn load_themes(&self, selected_path: Option<&str>, custom_path: Option<&str>) {
        self.loading.set(true);

        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        let active_path = active_theme_path();
        let env_themes = scan_env_themes(custom_path);
        *self.themes.borrow_mut() = env_themes.clone();

        for (idx, theme) in env_themes.iter().enumerate() {
            let row_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
            row_box.set_margin_top(8);
            row_box.set_margin_bottom(8);
            row_box.set_margin_start(12);
            row_box.set_margin_end(12);

            // Header row: name + active badge
            let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            header.set_valign(gtk4::Align::Center);

            let name_label = gtk4::Label::new(Some(&theme.name));
            name_label.set_halign(gtk4::Align::Start);
            name_label.set_css_classes(&["heading"]);
            name_label.set_hexpand(true);
            header.append(&name_label);

            let is_active = active_path.as_deref() == Some(theme.path.as_str())
                || selected_path == Some(theme.path.as_str());
            if is_active {
                let badge = gtk4::Label::new(Some("Active"));
                badge.set_css_classes(&["tag", "success"]);
                header.append(&badge);
            }

            let source_label = gtk4::Label::new(Some(&theme.source));
            source_label.set_css_classes(&["caption", "dim-label"]);
            source_label.set_halign(gtk4::Align::Start);

            row_box.append(&header);
            row_box.append(&source_label);

            // Color swatch
            let colors = theme.swatch_colors();
            if !colors.is_empty() {
                let swatch = gtk4::DrawingArea::new();
                swatch.set_size_request(-1, 16);
                swatch.set_vexpand(false);
                swatch.set_margin_top(4);
                let colors_clone = colors.clone();
                swatch.set_draw_func(move |_, cr, w, h| {
                    let n = colors_clone.len();
                    if n == 0 {
                        return;
                    }
                    let sw = w as f64 / n as f64;
                    for (i, (r, g, b)) in colors_clone.iter().enumerate() {
                        cr.set_source_rgb(*r, *g, *b);
                        cr.rectangle(i as f64 * sw, 0.0, sw, h as f64);
                        let _ = cr.fill();
                    }
                });
                row_box.append(&swatch);
            }

            self.list_box.append(&row_box);

            if is_active {
                if let Some(row) = self.list_box.row_at_index(idx as i32) {
                    self.list_box.select_row(Some(&row));
                }
            }
        }

        self.loading.set(false);
    }

    pub fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }
}

impl Default for ThemesPage {
    fn default() -> Self {
        use crate::model::settings::Settings;
        Self::new(Rc::new(RefCell::new(AppState::new(Settings::default()))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_valid() {
        let (r, g, b) = parse_hex_color("#cba6f7").unwrap();
        assert!((r - 0.796).abs() < 0.01);
        assert!((g - 0.651).abs() < 0.01);
        assert!((b - 0.969).abs() < 0.01);
    }

    #[test]
    fn test_parse_hex_color_invalid() {
        assert!(parse_hex_color("").is_none());
        assert!(parse_hex_color("#gggggg").is_none());
        assert!(parse_hex_color("#fff").is_none());
    }

    #[test]
    fn test_scan_env_themes_custom_path() {
        use std::fs;
        let dir = std::env::temp_dir().join("themes-test-scan");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("mytheme.env"),
            "COLOR_BASE=\"#1e1e2e\"\nFONT_SIZE=\"10\"\n",
        )
        .unwrap();

        let themes = scan_env_themes(Some(dir.to_str().unwrap()));
        assert!(
            themes.iter().any(|t| t.name == "mytheme"),
            "mytheme not found: {:?}",
            themes.iter().map(|t| &t.name).collect::<Vec<_>>()
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_env_themes_dedup() {
        use std::fs;
        let dir1 = std::env::temp_dir().join("themes-test-dedup1");
        fs::create_dir_all(&dir1).unwrap();
        fs::write(dir1.join("shared.env"), "COLOR_BASE=\"#111\"\n").unwrap();

        // scan_env_themes with custom path - theme appears once
        let themes = scan_env_themes(Some(dir1.to_str().unwrap()));
        let count = themes.iter().filter(|t| t.name == "shared").count();
        assert_eq!(count, 1, "should not duplicate theme");

        let _ = fs::remove_dir_all(&dir1);
    }

    #[test]
    fn test_theme_env_swatch_colors() {
        let mut vars = HashMap::new();
        vars.insert("COLOR_BASE".to_string(), "#1e1e2e".to_string());
        vars.insert("COLOR_TEXT".to_string(), "#cdd6f4".to_string());
        let theme = ThemeEnv {
            name: "test".into(),
            source: "test".into(),
            path: "".into(),
            vars,
        };
        let swatches = theme.swatch_colors();
        assert!(!swatches.is_empty());
    }
}
