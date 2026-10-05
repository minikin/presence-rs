//! Acceptance tests for spec 08: the JSON Schema of `Presence<T>`.

use presence_rs::Presence;
use schemars::generate::SchemaSettings;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "Patch")]
struct PresencePatch {
    #[serde(default, skip_serializing_if = "Presence::is_absent")]
    name: Presence<String>,
    id: u32,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "Patch")]
struct OptionPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    id: u32,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct Inner {
    x: i32,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct Nested {
    #[serde(default, skip_serializing_if = "Presence::is_absent")]
    inner: Presence<Inner>,
}

fn to_value(schema: schemars::Schema) -> Value {
    serde_json::to_value(schema).expect("a schema serializes to JSON")
}

fn required(schema: &Value) -> Vec<&str> {
    schema["required"]
        .as_array()
        .map(|names| names.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

#[test]
fn a_documented_presence_field_gets_the_schema_of_an_option_field() {
    // Given a struct with a documented Presence<String> field
    // And   the same struct with the field typed Option<String> and the same attributes
    // When  the JSON Schema of each is generated with schemars' default settings
    let presence = to_value(schema_for!(PresencePatch));
    let option = to_value(schema_for!(OptionPatch));

    // Then  the two schemas are equal
    assert_eq!(presence, option);
    // And   "name" has the schema {"type": ["string", "null"]}
    assert_eq!(
        presence["properties"]["name"],
        json!({"type": ["string", "null"]})
    );
    // And   "name" is not listed in "required"
    assert_eq!(required(&presence), ["id"]);
}

#[test]
fn the_field_stays_optional_when_the_schema_describes_serialization() {
    // Given the struct with the documented Presence field
    // When  its JSON Schema is generated for serialization
    let schema = to_value(
        SchemaSettings::default()
            .for_serialize()
            .into_generator()
            .into_root_schema_for::<PresencePatch>(),
    );

    // Then  "name" is not listed in "required"
    assert_eq!(required(&schema), ["id"]);
}

#[test]
fn the_openapi_3_0_settings_mark_the_field_nullable() {
    // Given the struct with the documented Presence field
    // When  its schema is generated with SchemaSettings::openapi3()
    let schema = to_value(
        SchemaSettings::openapi3()
            .into_generator()
            .into_root_schema_for::<PresencePatch>(),
    );

    // Then  "name" has the schema {"type": "string", "nullable": true}
    assert_eq!(
        schema["properties"]["name"],
        json!({"type": "string", "nullable": true})
    );
    // And   "name" is not listed in "required"
    assert_eq!(required(&schema), ["id"]);
}

#[test]
fn a_presence_of_a_referenced_type_adds_no_definition_of_its_own() {
    // Given a struct with a documented field inner: Presence<Inner>
    // When  its JSON Schema is generated
    let schema = to_value(schema_for!(Nested));

    // Then  "inner" is {"anyOf": [{"$ref": "#/$defs/Inner"}, {"type": "null"}]}
    assert_eq!(
        schema["properties"]["inner"],
        json!({"anyOf": [{"$ref": "#/$defs/Inner"}, {"type": "null"}]})
    );
    // And   "$defs" holds Inner and no definition named after Presence
    let definitions: Vec<&String> = schema["$defs"]
        .as_object()
        .expect("the schema has $defs")
        .keys()
        .collect();
    assert_eq!(definitions, ["Inner"]);
}
