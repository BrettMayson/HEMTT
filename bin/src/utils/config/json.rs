use serde_json;

use hemtt_config::{Class, Property, Value, Number, Item};

pub fn json_from_config_array(items: &[Item]) -> serde_json::Value {
    let mut values = Vec::new();
    for item in items {
        values.push(match item {
            Item::Str(value) => json_from_config_value(&Value::Str(value.clone())),
            Item::Number(value) => json_from_config_value(&Value::Number(value.clone())),
            Item::Array(sub_items) => json_from_config_array(sub_items),
            Item::Invalid(_) => serde_json::Value::Null
        });
    }
    serde_json::Value::Array(values)
}

pub fn json_from_config_value(value: &Value) -> serde_json::Value {
    match value {
        Value::Expression(expression) => serde_json::Value::String(expression.to_string()),
        Value::Invalid(_) => serde_json::Value::Null,
        Value::Str(str) => {
            let string = str.to_string();
            let raw_string = string.trim_matches(['"', '\\']);
            serde_json::Value::String(raw_string.into())
        },
        Value::UnexpectedArray(array) | Value::Array(array) => json_from_config_array(&array.items),
        Value::Number(number) => {
            let serde_number = match *number {
                Number::Int32 { value, .. } => serde_json::Number::from_i128(value.into()),
                Number::Int64 { value, .. } => serde_json::Number::from_i128(value.into()),
                Number::Float32 { value, .. } => serde_json::Number::from_f64(value.into()),
            }.expect("config property needs to be valid number");
            serde_json::Value::Number(serde_number)
        }
    }
}

pub fn json_from_config_property(property: &Property) -> (String, serde_json::Value) {
    match property {
        Property::Class(class) => json_from_config_class(class),
        Property::Entry { name, value, .. } => (name.value.clone(), json_from_config_value(value)),
        Property::Delete(name) | Property::MissingSemicolon(name, _) => (name.value.clone(), serde_json::Value::Null),
    }
}

pub fn json_from_config_class(class: &Class) -> (String, serde_json::Value) {
    match class {
        Class::Root { properties} => {
            let mut value_map = serde_json::Map::new();
            for root_property in properties {
                let (key, value) = json_from_config_property(root_property);
                value_map.insert(key, value);
            }
            (String::from("__root__"), serde_json::Value::Object(value_map))
        }
        Class::Local { name, properties, .. } => {
            let mut value_map = serde_json::Map::new();
            for root_property in properties {
                let (key, value) = json_from_config_property(root_property);
                value_map.insert(key, value);
            }
            (name.to_string(), serde_json::Value::Object(value_map))
        },
        Class::External { name } => (name.to_string(), serde_json::Value::Null),
    }
}
