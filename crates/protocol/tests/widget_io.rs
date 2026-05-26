use openwebhmi_protocol::ExportedWidget;

#[test]
fn exported_widget_round_trips_json() {
    let encoded = serde_json::json!({
        "schema_version": 1,
        "widget_type": "AlarmTable",
        "props": { "title": "Critical alarms", "pageSize": 20 },
        "bindings": [],
        "children": [{
            "widget_type": "Button",
            "props": { "label": "Ack" },
            "bindings": [],
            "children": []
        }],
        "exported_at": 1_779_711_200_000u64,
        "openwebhmi_version": "0.0.1"
    });

    let decoded: ExportedWidget =
        serde_json::from_value(encoded.clone()).expect("test fixture deserializes");
    let reencoded = serde_json::to_value(decoded).expect("test fixture serializes");

    assert_eq!(reencoded, encoded);
}
