use sway_config::config::detect;
/// Tests for Sway session detection and configuration parsing
/// These tests use fixture JSON strings and do NOT invoke the live swaymsg command
use sway_config::config::detect::{LibinputConfig, Rect, SwayInput, SwayOutput};
use sway_config::config::swaymsg::is_sway_running;
use sway_config::model::input::{KeyboardConfig, TouchpadConfig};
use sway_config::model::output::OutputConfig;

// ============================================================================
// JSON FIXTURES
// ============================================================================

const SWAY_OUTPUT_FIXTURE: &str = r#"[{"id":1,"name":"eDP-1","rect":{"x":0,"y":0,"width":1920,"height":1080},"focused":true,"active":true,"dpms":true,"primary":false,"make":"AU Optronics","model":"0x1420","serial":"0x00000000","scale":1.0,"transform":"normal","current_mode":{"width":1920,"height":1080,"refresh":60000},"modes":[{"width":1920,"height":1080,"refresh":60000}]}]"#;

const SWAY_KEYBOARD_FIXTURE: &str = r#"[{"identifier":"1:1:AT_Translated_Set_2_keyboard","name":"AT Translated Set 2 keyboard","vendor":1,"product":1,"type":"keyboard","xkb_active_layout_name":"English (UK)","xkb_layouts_as_symbols":["gb"]}]"#;

const SWAY_TOUCHPAD_FIXTURE: &str = r#"[{"identifier":"2:7:SynPS/2_Synaptics_TouchPad","name":"SynPS/2 Synaptics TouchPad","vendor":2,"product":7,"type":"touchpad","libinput":{"send_events":"enabled","tap":"enabled","natural_scroll":"disabled","dwt":"enabled","accel_speed":0.0,"accel_profile":"adaptive","left_handed":"disabled","middle_emulation":"disabled"}}]"#;

const EMPTY_OUTPUTS_FIXTURE: &str = "[]";

const EMPTY_INPUTS_FIXTURE: &str = "[]";

// ============================================================================
// PARSER TESTS - JSON DESERIALIZATION
// ============================================================================

#[test]
fn test_parse_sway_output_from_json() {
    let outputs: Vec<SwayOutput> =
        serde_json::from_str(SWAY_OUTPUT_FIXTURE).expect("Failed to parse output fixture");

    assert_eq!(outputs.len(), 1);
    let output = &outputs[0];

    assert_eq!(output.name, "eDP-1");
    assert_eq!(output.make, "AU Optronics");
    assert_eq!(output.model, "0x1420");
    assert_eq!(output.serial, "0x00000000");
    assert!(output.active);
    assert!(output.dpms);
    assert!(!output.primary);
    assert_eq!(output.scale, 1.0);
    assert_eq!(output.transform, "normal");
    assert!(output.focused);

    assert_eq!(output.rect.x, 0);
    assert_eq!(output.rect.y, 0);
    assert_eq!(output.rect.width, 1920);
    assert_eq!(output.rect.height, 1080);

    assert!(output.current_mode.is_some());
    let mode = output.current_mode.as_ref().unwrap();
    assert_eq!(mode.width, 1920);
    assert_eq!(mode.height, 1080);
    assert_eq!(mode.refresh, 60000);

    assert_eq!(output.modes.len(), 1);
}

#[test]
fn test_parse_sway_input_keyboard_from_json() {
    let inputs: Vec<SwayInput> =
        serde_json::from_str(SWAY_KEYBOARD_FIXTURE).expect("Failed to parse keyboard fixture");

    assert_eq!(inputs.len(), 1);
    let input = &inputs[0];

    assert_eq!(input.identifier, "1:1:AT_Translated_Set_2_keyboard");
    assert_eq!(input.name, "AT Translated Set 2 keyboard");
    assert_eq!(input.vendor, 1);
    assert_eq!(input.product, 1);
    assert_eq!(input.type_, "keyboard");
    assert_eq!(
        input.xkb_active_layout_name,
        Some("English (UK)".to_string())
    );
    assert!(input.libinput.is_none());
}

#[test]
fn test_parse_sway_input_touchpad_from_json() {
    let inputs: Vec<SwayInput> =
        serde_json::from_str(SWAY_TOUCHPAD_FIXTURE).expect("Failed to parse touchpad fixture");

    assert_eq!(inputs.len(), 1);
    let input = &inputs[0];

    assert_eq!(input.identifier, "2:7:SynPS/2_Synaptics_TouchPad");
    assert_eq!(input.name, "SynPS/2 Synaptics TouchPad");
    assert_eq!(input.vendor, 2);
    assert_eq!(input.product, 7);
    assert_eq!(input.type_, "touchpad");

    let libinput = input
        .libinput
        .as_ref()
        .expect("touchpad should have libinput config");
    assert_eq!(libinput.send_events, Some("enabled".to_string()));
    assert_eq!(libinput.tap, Some("enabled".to_string()));
    assert_eq!(libinput.natural_scroll, Some("disabled".to_string()));
    assert_eq!(libinput.dwt, Some("enabled".to_string()));
    assert_eq!(libinput.accel_speed, Some(0.0));
    assert_eq!(libinput.accel_profile, Some("adaptive".to_string()));
    assert_eq!(libinput.left_handed, Some("disabled".to_string()));
    assert_eq!(libinput.middle_emulation, Some("disabled".to_string()));
}

#[test]
fn test_parse_empty_outputs() {
    let outputs: Vec<SwayOutput> =
        serde_json::from_str(EMPTY_OUTPUTS_FIXTURE).expect("Failed to parse empty outputs");
    assert_eq!(outputs.len(), 0);
}

#[test]
fn test_parse_empty_inputs() {
    let inputs: Vec<SwayInput> =
        serde_json::from_str(EMPTY_INPUTS_FIXTURE).expect("Failed to parse empty inputs");
    assert_eq!(inputs.len(), 0);
}

// ============================================================================
// DETECTION TESTS - FILTERING
// ============================================================================

#[test]
fn test_detect_keyboards_filters_correctly() {
    // Create mixed input types
    let keyboard_input = SwayInput {
        identifier: "test_kb".to_string(),
        name: "Test Keyboard".to_string(),
        vendor: 1,
        product: 1,
        type_: "keyboard".to_string(),
        xkb_active_layout_name: Some("us".to_string()),
        xkb_layouts_as_symbols: vec!["us".to_string()],
        libinput: None,
        repeat_delay: None,
        repeat_rate: None,
    };

    let touchpad_input = SwayInput {
        identifier: "test_tp".to_string(),
        name: "Test Touchpad".to_string(),
        vendor: 2,
        product: 2,
        type_: "touchpad".to_string(),
        xkb_active_layout_name: None,
        xkb_layouts_as_symbols: vec![],
        repeat_delay: None,
        repeat_rate: None,
        libinput: Some(LibinputConfig {
            send_events: None,
            tap: None,
            natural_scroll: None,
            dwt: None,
            accel_speed: None,
            accel_profile: None,
            left_handed: None,
            middle_emulation: None,
        }),
    };

    let inputs = vec![keyboard_input, touchpad_input];
    let keyboards = detect::detect_keyboards(&inputs);

    assert_eq!(keyboards.len(), 1);
    assert_eq!(keyboards[0].type_, "keyboard");
}

#[test]
fn test_detect_touchpads_filters_correctly() {
    let keyboard_input = SwayInput {
        identifier: "test_kb".to_string(),
        name: "Test Keyboard".to_string(),
        vendor: 1,
        product: 1,
        type_: "keyboard".to_string(),
        xkb_active_layout_name: Some("us".to_string()),
        xkb_layouts_as_symbols: vec!["us".to_string()],
        libinput: None,
        repeat_delay: None,
        repeat_rate: None,
    };

    let touchpad_input = SwayInput {
        identifier: "test_tp".to_string(),
        name: "Test Touchpad".to_string(),
        vendor: 2,
        product: 2,
        type_: "touchpad".to_string(),
        xkb_active_layout_name: None,
        xkb_layouts_as_symbols: vec![],
        repeat_delay: None,
        repeat_rate: None,
        libinput: Some(LibinputConfig {
            send_events: None,
            tap: None,
            natural_scroll: None,
            dwt: None,
            accel_speed: None,
            accel_profile: None,
            left_handed: None,
            middle_emulation: None,
        }),
    };

    let inputs = vec![keyboard_input, touchpad_input];
    let touchpads = detect::detect_touchpads(&inputs);

    assert_eq!(touchpads.len(), 1);
    assert_eq!(touchpads[0].type_, "touchpad");
}

#[test]
fn test_detect_touchpads_by_name() {
    let touchpad_by_name = SwayInput {
        identifier: "test_custom".to_string(),
        name: "My Custom Touchpad Device".to_string(),
        vendor: 99,
        product: 99,
        type_: "other".to_string(),
        xkb_active_layout_name: None,
        xkb_layouts_as_symbols: vec![],
        repeat_delay: None,
        repeat_rate: None,
        libinput: Some(LibinputConfig {
            send_events: None,
            tap: None,
            natural_scroll: None,
            dwt: None,
            accel_speed: None,
            accel_profile: None,
            left_handed: None,
            middle_emulation: None,
        }),
    };

    let inputs = vec![touchpad_by_name];
    let touchpads = detect::detect_touchpads(&inputs);

    assert_eq!(touchpads.len(), 1, "Should detect touchpad by name");
}

// ============================================================================
// MODEL CONVERSION TESTS
// ============================================================================

#[test]
fn test_output_config_from_sway() {
    let sway_output: SwayOutput = serde_json::from_str(SWAY_OUTPUT_FIXTURE)
        .map(|v: Vec<SwayOutput>| v.into_iter().next().unwrap())
        .expect("Failed to parse fixture");

    let config = OutputConfig::from_sway(&sway_output);

    assert_eq!(config.name, "eDP-1");
    assert!(config.enabled);
    assert_eq!(config.scale, 1.0);
    assert_eq!(config.position.x, 0);
    assert_eq!(config.position.y, 0);
    assert_eq!(
        config.transform,
        sway_config::model::output::Transform::Normal
    );

    assert!(config.resolution.is_some());
    let res = config.resolution.unwrap();
    assert_eq!(res.width, 1920);
    assert_eq!(res.height, 1080);
}

#[test]
fn test_keyboard_config_from_sway() {
    let sway_input: SwayInput = serde_json::from_str(SWAY_KEYBOARD_FIXTURE)
        .map(|v: Vec<SwayInput>| v.into_iter().next().unwrap())
        .expect("Failed to parse fixture");

    let config = KeyboardConfig::from_sway(&sway_input);

    // Layout and options come from the sway config file; variant is empty (not set in sway config).
    let (sway_layout, sway_variant, sway_options) =
        sway_config::config::detect::sway_config_keyboard_defaults();
    assert_eq!(config.identifier, "1:1:AT_Translated_Set_2_keyboard");
    assert_eq!(
        config.xkb_layout,
        if !sway_layout.is_empty() {
            sway_layout
        } else {
            "gb".to_string()
        }
    );
    assert_eq!(config.xkb_variant, sway_variant);
    assert_eq!(config.xkb_options, sway_options);
    assert_eq!(config.repeat_delay, 600);
    assert_eq!(config.repeat_rate, 25);
}

#[test]
fn test_touchpad_config_from_sway() {
    let sway_input: SwayInput = serde_json::from_str(SWAY_TOUCHPAD_FIXTURE)
        .map(|v: Vec<SwayInput>| v.into_iter().next().unwrap())
        .expect("Failed to parse fixture");

    let config = TouchpadConfig::from_sway(&sway_input);

    assert_eq!(config.identifier, "2:7:SynPS/2_Synaptics_TouchPad");

    // Based on fixture: tap="enabled", natural_scroll="disabled", etc.
    assert!(config.tap_to_click); // enabled
    assert!(!config.natural_scroll); // disabled
    assert!(config.dwt); // enabled
    assert_eq!(config.accel_speed, 0.0);
    assert_eq!(
        config.accel_profile,
        sway_config::model::input::AccelProfile::Adaptive
    );
    assert!(!config.left_handed); // disabled
    assert!(!config.middle_emulation); // disabled
}

#[test]
fn test_output_config_defaults() {
    let sway_output = SwayOutput {
        name: "test".to_string(),
        make: "".to_string(),
        model: "".to_string(),
        serial: "".to_string(),
        active: false,
        dpms: false,
        primary: false,
        rect: Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        current_mode: None,
        modes: vec![],
        scale: 1.0,
        transform: "normal".to_string(),
        focused: false,
    };

    let config = OutputConfig::from_sway(&sway_output);

    assert_eq!(config.scale, 1.0);
    assert_eq!(
        config.transform,
        sway_config::model::output::Transform::Normal
    );
}

#[test]
fn test_keyboard_config_defaults() {
    let sway_input = SwayInput {
        identifier: "test".to_string(),
        name: "test keyboard".to_string(),
        vendor: 0,
        product: 0,
        type_: "keyboard".to_string(),
        xkb_active_layout_name: None,
        xkb_layouts_as_symbols: vec![],
        repeat_delay: None,
        repeat_rate: None,
        libinput: None,
    };

    let config = KeyboardConfig::from_sway(&sway_input);

    // Layout comes from sway config or /etc/default/keyboard fallback.
    let (expected_layout, _) = sway_config::config::detect::system_keyboard_layout();
    let (sway_layout, _, _) = sway_config::config::detect::sway_config_keyboard_defaults();
    let expected = if !sway_layout.is_empty() {
        sway_layout
    } else {
        expected_layout
    };
    assert_eq!(config.xkb_layout, expected);
    assert_eq!(config.repeat_delay, 600);
    assert_eq!(config.repeat_rate, 25);
}

#[test]
fn test_transform_default_is_normal() {
    use sway_config::model::output::Transform;
    assert_eq!(Transform::default(), Transform::Normal);
}

#[test]
fn test_is_sway_running_returns_bool() {
    // This test just verifies the function exists and returns a bool
    // It doesn't require Sway to be running
    let _result = is_sway_running();
    // If we get here without panic, the test passes
}
