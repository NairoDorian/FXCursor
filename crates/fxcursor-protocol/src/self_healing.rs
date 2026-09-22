use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

const MAX_REPAIR_ROUNDS: usize = 64;

#[derive(Debug)]
pub struct RepairOutcome<T> {
    pub value: T,
    pub repaired_paths: Vec<String>,
    pub needs_rewrite: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum PathSegment {
    Key(String),
    Index(usize),
}

fn parse_path(path: &str) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut chars = path.chars().peekable();
    let mut current_key = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '.' => {
                if !current_key.is_empty() {
                    segments.push(PathSegment::Key(std::mem::take(&mut current_key)));
                }
            }
            '[' => {
                if !current_key.is_empty() {
                    segments.push(PathSegment::Key(std::mem::take(&mut current_key)));
                }
                let mut index_str = String::new();
                for inner in chars.by_ref() {
                    if inner == ']' {
                        break;
                    }
                    index_str.push(inner);
                }
                if let Ok(idx) = index_str.parse::<usize>() {
                    segments.push(PathSegment::Index(idx));
                }
            }
            _ => {
                current_key.push(ch);
            }
        }
    }

    if !current_key.is_empty() {
        segments.push(PathSegment::Key(current_key));
    }

    segments
}

fn merge_defaults(
    target: &mut Value,
    default: &Value,
    current_path: &str,
    repaired_paths: &mut Vec<String>,
) -> bool {
    let mut modified = false;

    match (target, default) {
        (Value::Object(target_map), Value::Object(default_map)) => {
            for (k, def_val) in default_map {
                let sub_path = if current_path.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", current_path, k)
                };
                if let Some(target_val) = target_map.get_mut(k) {
                    if merge_defaults(target_val, def_val, &sub_path, repaired_paths) {
                        modified = true;
                    }
                } else {
                    target_map.insert(k.clone(), def_val.clone());
                    repaired_paths.push(format!("(missing) {}", sub_path));
                    modified = true;
                }
            }
        }
        (t, d) => {
            if std::mem::discriminant(t) != std::mem::discriminant(d) {
                *t = d.clone();
                repaired_paths.push(current_path.to_string());
                modified = true;
            }
        }
    }

    modified
}

fn reset_path(target: &mut Value, default: &Value, segments: &[PathSegment]) -> bool {
    if segments.is_empty() {
        *target = default.clone();
        return true;
    }

    match &segments[0] {
        PathSegment::Key(k) => {
            if let (Value::Object(t_map), Value::Object(d_map)) = (target, default)
                && let Some(d_val) = d_map.get(k)
            {
                let t_val = t_map.entry(k.clone()).or_insert_with(|| d_val.clone());
                return reset_path(t_val, d_val, &segments[1..]);
            }
        }
        PathSegment::Index(idx) => {
            if let (Value::Array(t_arr), Value::Array(d_arr)) = (target, default)
                && *idx < d_arr.len()
            {
                while t_arr.len() <= *idx {
                    t_arr.push(d_arr[t_arr.len()].clone());
                }
                return reset_path(&mut t_arr[*idx], &d_arr[*idx], &segments[1..]);
            }
        }
    }

    false
}

pub fn deserialize_with_self_healing<T>(raw_json: &str) -> Result<RepairOutcome<T>, String>
where
    T: Serialize + DeserializeOwned + Default,
{
    let mut parsed_value: Value = match serde_json::from_str(raw_json) {
        Ok(v) => v,
        Err(_) => {
            let default_val = T::default();
            return Ok(RepairOutcome {
                value: default_val,
                repaired_paths: vec!["<entire document corrupted>".to_string()],
                needs_rewrite: true,
            });
        }
    };

    let default_instance = T::default();
    let default_value = serde_json::to_value(&default_instance)
        .map_err(|e| format!("Failed to serialize default instance: {}", e))?;

    let mut repaired_paths = Vec::new();
    let mut needs_rewrite =
        merge_defaults(&mut parsed_value, &default_value, "", &mut repaired_paths);

    for _ in 0..MAX_REPAIR_ROUNDS {
        let deserializer = parsed_value.clone();
        let result: Result<T, _> = serde_path_to_error::deserialize(deserializer);

        match result {
            Ok(val) => {
                return Ok(RepairOutcome {
                    value: val,
                    repaired_paths,
                    needs_rewrite,
                });
            }
            Err(err) => {
                let path_str = err.path().to_string();
                log::warn!(
                    "[self-healing] Broken config field at '{}': {}",
                    path_str,
                    err
                );

                let segments = parse_path(&path_str);
                reset_path(&mut parsed_value, &default_value, &segments);

                repaired_paths.push(path_str);
                needs_rewrite = true;
            }
        }
    }

    log::error!("[self-healing] Exceeded max repair rounds; returning clean default config.");
    Ok(RepairOutcome {
        value: default_instance,
        repaired_paths,
        needs_rewrite: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn test_valid_json_deserializes_cleanly() {
        let default_config = AppConfig::default();
        let json_str = serde_json::to_string_pretty(&default_config).unwrap();
        let outcome: RepairOutcome<AppConfig> = deserialize_with_self_healing(&json_str).unwrap();

        assert_eq!(outcome.value, default_config);
        assert!(outcome.repaired_paths.is_empty());
        assert!(!outcome.needs_rewrite);
    }

    #[test]
    fn test_heals_corrupted_field_without_losing_rest() {
        let json_with_bad_field = r#"{
            "enabled": true,
            "trail": {
                "length": "this should be a number, not a string"
            }
        }"#;

        let outcome: RepairOutcome<AppConfig> =
            deserialize_with_self_healing(json_with_bad_field).unwrap();

        assert!(outcome.needs_rewrite);
        assert!(!outcome.repaired_paths.is_empty());
        assert!(outcome.value.enabled);
        assert_eq!(outcome.value.trail.length, 80); // restored default
    }

    #[test]
    fn test_heals_completely_invalid_syntax() {
        let bad_syntax = "{ not: valid json at all ...";
        let outcome: RepairOutcome<AppConfig> = deserialize_with_self_healing(bad_syntax).unwrap();
        assert!(outcome.needs_rewrite);
        assert_eq!(outcome.value, AppConfig::default());
        assert_eq!(outcome.repaired_paths, vec!["<entire document corrupted>"]);
    }

    #[test]
    fn test_heals_array_length_mismatch() {
        let partial_array = r#"{
            "enabled": true,
            "trail": {
                "layers": [
                    { "enabled": false }
                ]
            }
        }"#;
        let outcome: RepairOutcome<AppConfig> =
            deserialize_with_self_healing(partial_array).unwrap();
        assert!(outcome.needs_rewrite);
        assert_eq!(outcome.value.trail.layers.len(), 4);
    }
}
