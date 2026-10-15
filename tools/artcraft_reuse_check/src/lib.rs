#[cfg(test)]
mod tests {
    use craft_ui::docking::{Action, Layout, Node};
    use serde::{Deserialize, Serialize};

    #[test]
    fn workspace_moves_roundtrip_and_invalid_moves_preserve_panels() {
        let mut layout = Layout {
            root: Some(Node::Tabs { panels: vec!["scene", "network", "parameters"], active: 0 }),
            floating: vec![],
        };
        layout.apply(Action::Float { panel: "parameters", rect: [20., 20., 300., 600.] }).unwrap();
        let encoded = serde_json::to_string(&layout).unwrap();
        let mut restored: Layout<String> = serde_json::from_str(&encoded).unwrap();
        restored.validate().unwrap();
        let before = restored.clone();
        assert!(restored.apply(Action::Activate { panel: "missing".into() }).is_err());
        assert_eq!(before, restored);
        assert_eq!(restored.floating[0].panels, vec!["parameters"]);
    }

    #[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    struct Prefs { appearance: Appearance }
    #[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    struct Appearance { theme: String }
    impl craft_settings::Settings for Prefs {
        const APP_ID: &'static str = "lokicraft-reuse-smoke";
        const SCHEMA_VERSION: u32 = 1;
    }

    #[test]
    fn settings_can_run_without_touching_user_profile() {
        let (mut store, _) = craft_settings::SettingsStore::<Prefs>::open(craft_settings::Location::Memory);
        store.update(|p| p.appearance.theme = "houdini-reference".into());
        store.flush().unwrap();
        assert!(!store.is_persistent());
        assert!(store.path().is_none());
    }

    #[test]
    fn failed_save_preserves_previous_document() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("save-fixture.tmp");
        craft_atomic::atomic_write(&path, b"original").unwrap();
        let result = craft_atomic::atomic_write_with(&path, |file| {
            use std::io::Write;
            file.write_all(b"partial")?;
            Err(std::io::Error::other("injected failure"))
        });
        assert!(result.is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"original");
        std::fs::remove_file(path).unwrap();
    }
}
