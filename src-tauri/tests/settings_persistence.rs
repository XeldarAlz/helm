use std::fs;

use helm_lib::models::settings::AppSettings;

#[test]
fn settings_roundtrip_via_serde() {
    let original = AppSettings::default();
    let json = serde_json::to_string_pretty(&original).expect("serialize");
    let restored: AppSettings = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(original.project_dir, restored.project_dir);
    assert_eq!(original.claude_cli_path, restored.claude_cli_path);
    assert_eq!(original.max_concurrent_agents, restored.max_concurrent_agents);
    assert_eq!(original.font_size, restored.font_size);
    assert_eq!(original.sidebar_collapsed, restored.sidebar_collapsed);
    assert_eq!(original.theme, restored.theme);
    assert_eq!(original.reduced_motion, restored.reduced_motion);
}

#[test]
fn settings_write_and_read_from_tempfile() {
    let dir = std::env::temp_dir().join("helm_test_settings");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let path = dir.join("settings.json");
    let original = AppSettings::default();
    let json = serde_json::to_string_pretty(&original).unwrap();

    // Atomic write pattern: tmp → rename
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &json).unwrap();
    fs::rename(&tmp, &path).unwrap();

    // Read back
    let read_json = fs::read_to_string(&path).unwrap();
    let restored: AppSettings = serde_json::from_str(&read_json).unwrap();

    assert_eq!(original.claude_cli_path, restored.claude_cli_path);
    assert_eq!(original.theme, restored.theme);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn settings_corrupt_json_falls_back_to_default() {
    let result: Result<AppSettings, _> = serde_json::from_str("{ broken json }}}");
    assert!(result.is_err());
    // The app code uses unwrap_or_default for this case
    let settings = result.unwrap_or_default();
    assert_eq!(settings.claude_cli_path, "claude");
}
