use openwebhmi_protocol::{ClientMessage, DriverId, Quality, ServerMessage, TagPath, TagValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Identifiers {
    tag_path: TagPath,
    driver_id: DriverId,
}

#[test]
fn identifier_newtypes_are_transparent_json_strings() {
    let identifiers = Identifiers {
        tag_path: TagPath::new("rockwell-1/Pressure"),
        driver_id: DriverId::new("rockwell-1"),
    };

    let json = serde_json::to_string(&identifiers).unwrap();
    assert_eq!(
        json,
        r#"{"tag_path":"rockwell-1/Pressure","driver_id":"rockwell-1"}"#
    );
    assert_eq!(
        serde_json::from_str::<Identifiers>(&json).unwrap(),
        identifiers
    );
}

#[test]
fn tag_write_json_shape_matches_pre_newtype_wire_form() {
    let message = ClientMessage::TagWrite {
        path: TagPath::new("a/b"),
        value: TagValue::Real(12.5),
    };

    assert_eq!(
        serde_json::to_string(&message).unwrap(),
        r#"{"kind":"tag.write","path":"a/b","value":{"type":"real","value":12.5}}"#
    );
}

#[test]
fn tag_path_and_driver_id_are_not_interchangeable_types() {
    fn tag_path_type(_: TagPath) -> &'static str {
        "tag"
    }
    fn driver_id_type(_: DriverId) -> &'static str {
        "driver"
    }

    assert_eq!(tag_path_type(TagPath::new("driver/tag")), "tag");
    assert_eq!(driver_id_type(DriverId::new("driver")), "driver");
    assert_ne!(
        std::any::TypeId::of::<TagPath>(),
        std::any::TypeId::of::<DriverId>()
    );
}

#[test]
fn non_exhaustive_messages_are_matched_with_wildcard_arms() {
    let client = ClientMessage::Ping;
    let client_kind = match client {
        ClientMessage::Ping => "ping",
        _ => "other",
    };
    assert_eq!(client_kind, "ping");

    let server = ServerMessage::TagUpdate {
        path: TagPath::new("system/sim/sin"),
        value: TagValue::Real(0.0),
        quality: Quality::Good,
        ts: 1,
    };
    let server_kind = match server {
        ServerMessage::TagUpdate { .. } => "tag.update",
        _ => "other",
    };
    assert_eq!(server_kind, "tag.update");
}
