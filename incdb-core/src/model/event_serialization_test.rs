//! Event Serialization/Deserialization Test
//!
//! Eventのシリアライゼーション/デシリアライゼーションが正しく動作するか確認

use crate::model::{Event, EventType, IId, Incidence, Level, RoleId, Value};
use bincode;

#[test]
fn test_event_serialization_roundtrip() {
    // Eventを作成（related_entitiesを含む）
    let event = Event::new(
        IId(1),
        1234567890000i64,
        EventType::PersonCreated,
        IId(2),
        Value::string("John Doe"),
    )
    .add_related_entity(IId(3), RoleId(1))
    .add_related_entity(IId(4), RoleId(2));

    // シリアライズ
    let serialized = bincode::serialize(&event).expect("Failed to serialize");
    
    // デシリアライズ
    let deserialized: Event = bincode::deserialize(&serialized).expect("Failed to deserialize");

    // 検証
    assert_eq!(event.id, deserialized.id);
    assert_eq!(event.timestamp, deserialized.timestamp);
    assert_eq!(event.entity_id, deserialized.entity_id);
    assert_eq!(event.related_entities.len(), deserialized.related_entities.len());
    assert_eq!(event.related_entities, deserialized.related_entities);
}

#[test]
fn test_incidence_to_event_to_incidence_roundtrip() {
    // Incidenceを作成（argsとrolesを含む）
    let original_incidence = Incidence::new(IId(1), Level::zero())
        .with_val(Value::string("Test"))
        .add_arg(IId(2), RoleId(1))
        .add_arg(IId(3), RoleId(2));

    // Incidence -> Event
    let event: Event = original_incidence.clone().into();
    
    // Eventをシリアライズ
    let serialized = bincode::serialize(&event).expect("Failed to serialize");
    
    // デシリアライズ
    let deserialized_event: Event = bincode::deserialize(&serialized).expect("Failed to deserialize");
    
    // Event -> Incidence
    let restored_incidence: Incidence = deserialized_event.into();

    // 検証
    assert_eq!(original_incidence.id, restored_incidence.id);
    assert_eq!(original_incidence.args.len(), restored_incidence.args.len());
    assert_eq!(original_incidence.roles.len(), restored_incidence.roles.len());
    assert_eq!(original_incidence.args, restored_incidence.args);
    assert_eq!(original_incidence.roles, restored_incidence.roles);
}

#[test]
fn test_event_with_empty_related_entities() {
    // related_entitiesが空のEvent
    let event = Event::new(
        IId(1),
        1234567890000i64,
        EventType::PersonCreated,
        IId(2),
        Value::string("John Doe"),
    );

    // シリアライズ/デシリアライズ
    let serialized = bincode::serialize(&event).expect("Failed to serialize");
    let deserialized: Event = bincode::deserialize(&serialized).expect("Failed to deserialize");

    // 検証
    assert_eq!(event.related_entities.len(), deserialized.related_entities.len());
    assert!(deserialized.related_entities.is_empty());
}

#[test]
fn test_event_with_multiple_related_entities() {
    // 複数のrelated_entitiesを持つEvent
    let event = Event::new(
        IId(1),
        1234567890000i64,
        EventType::PersonCreated,
        IId(2),
        Value::string("John Doe"),
    )
    .add_related_entity(IId(3), RoleId(1))
    .add_related_entity(IId(4), RoleId(2))
    .add_related_entity(IId(5), RoleId(3));

    // シリアライズ/デシリアライズ
    let serialized = bincode::serialize(&event).expect("Failed to serialize");
    let deserialized: Event = bincode::deserialize(&serialized).expect("Failed to deserialize");

    // 検証
    assert_eq!(event.related_entities.len(), 3);
    assert_eq!(deserialized.related_entities.len(), 3);
    assert_eq!(event.related_entities, deserialized.related_entities);
}
