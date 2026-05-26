use openwebhmi_project_store::{ArtifactKind, ProjectStore};
use serde_json::json;

#[test]
fn theme_artifact_round_trips_as_project_singleton() {
    let tempdir = tempfile::tempdir().unwrap();
    let store = ProjectStore::open(tempdir.path()).unwrap();
    store
        .save_artifact(
            "demo",
            ArtifactKind::ProjectMeta,
            json!({ "schema_version": 1, "name": "Demo" }),
        )
        .unwrap();

    let first = theme("#ff0000");
    let second = theme("#00ff00");
    store
        .save_artifact("demo", ArtifactKind::Theme, first)
        .unwrap();
    store
        .save_artifact("demo", ArtifactKind::Theme, second.clone())
        .unwrap();

    assert_eq!(
        store.read_artifact("demo", ArtifactKind::Theme).unwrap(),
        Some(second)
    );
    assert_eq!(
        store
            .load("demo")
            .unwrap()
            .theme
            .unwrap()
            .light
            .primary_color,
        "#00ff00"
    );
}

fn theme(primary: &str) -> serde_json::Value {
    json!({
        "mode": "light",
        "light": variables(primary, "#ffffff", "#1f2933"),
        "dark": variables("#60a5fa", "#111827", "#f9fafb")
    })
}

fn variables(primary: &str, background: &str, text: &str) -> serde_json::Value {
    json!({
        "primary_color": primary,
        "secondary_color": "#52606d",
        "background": background,
        "surface": "#ffffff",
        "text_primary": text,
        "text_secondary": "#52606d",
        "accent": "#2563eb",
        "error": "#dc2626",
        "warning": "#f59e0b",
        "font_family": "ui-sans-serif, system-ui, sans-serif",
        "font_size_base": 14,
        "spacing_unit": 8,
        "border_radius": 6
    })
}
