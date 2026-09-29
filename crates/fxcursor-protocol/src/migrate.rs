//! One-off conversion of configurations written before the parameter units changed.
//!
//! Springs used to be stored as `spring / 1000` (a 1–500 integer-ish scale) and frictions as a
//! percentage of velocity *lost*. Both are now plain fractions in `0.0..1.0`. A config file
//! written by an older build therefore holds values that are wildly out of the new range
//! (`spring: 50`, `damping: 30`) and would be silently clamped to a dead trail, so it is
//! converted on load instead.
//!
//! The version counter in [`AppConfig::schema_version`] is what marks a file as pre- or
//! post-migration. Bump it whenever another conversion is added.

use crate::config::AppConfig;
use serde_json::Value;

/// Schema version this build writes. Bump when adding a conversion to [`migrate_value`].
pub const CURRENT_SCHEMA_VERSION: u32 = 2;

/// First version that stored springs and frictions as plain `0.0..1.0` fractions.
const NATURAL_UNITS_VERSION: u32 = 2;

/// Converts an older configuration in place and returns `true` when something changed.
///
/// Must run **before** [`AppConfig::sanitize`], which would otherwise clamp the legacy
/// values to the new bounds and lose them.
pub fn migrate(config: &mut AppConfig) -> bool {
    let mut changed = false;
    if config.schema_version < NATURAL_UNITS_VERSION {
        convert_to_natural_units(config);
        changed = true;
    }
    if config.schema_version != CURRENT_SCHEMA_VERSION {
        config.schema_version = CURRENT_SCHEMA_VERSION;
        changed = true;
    }
    changed
}

/// [`migrate`] on the raw JSON document, for the load path.
///
/// Works on the parsed value rather than the typed struct because a pre-migration file has no
/// `schema_version` field at all: deserializing it first would fill that field from the
/// current default and the conversion would be skipped.
pub fn migrate_value(value: &mut Value) -> bool {
    let Some(root) = value.as_object_mut() else {
        return false;
    };
    let version = root
        .get("schema_version")
        .and_then(Value::as_u64)
        .unwrap_or(1) as u32;
    if version >= CURRENT_SCHEMA_VERSION {
        return false;
    }

    if version < NATURAL_UNITS_VERSION {
        if let Some(trail) = root.get_mut("trail").and_then(Value::as_object_mut) {
            for key in ["spring", "head_spring"] {
                if let Some(v) = trail.get(key).and_then(Value::as_f64) {
                    *trail.entry(key).or_insert(Value::Null) = json_f64(v / 1000.0);
                }
            }
            for key in ["damping", "head_damping"] {
                if let Some(v) = trail.get(key).and_then(Value::as_f64) {
                    *trail.entry(key).or_insert(Value::Null) = json_f64(1.0 - v / 100.0);
                }
            }
        }
        if let Some(head) = root.get_mut("head").and_then(Value::as_object_mut) {
            for key in ["squish_intensity", "squish_smoothing"] {
                if let Some(v) = head.get(key).and_then(Value::as_f64) {
                    *head.entry(key).or_insert(Value::Null) = json_f64(v / 100.0);
                }
            }
        }
    }

    root.insert(
        "schema_version".to_string(),
        Value::from(CURRENT_SCHEMA_VERSION),
    );
    true
}

/// `Value::from(f64)` needs a finite number; a NaN in the source would panic.
fn json_f64(v: f64) -> Value {
    if v.is_finite() {
        Value::from(v)
    } else {
        Value::from(0.0)
    }
}

/// Loads an `AppConfig` document, converting older parameter units on the way.
///
/// The generic self-healing deserializer cannot do this itself: it fills missing fields from the
/// *current* defaults before the typed struct is built, which would stamp the current
/// `schema_version` onto an old document and make the conversion a no-op. Working on the raw
/// JSON first is the only order that sees the file as the user left it.
pub fn deserialize_config(raw_json: &str) -> Result<crate::RepairOutcome<AppConfig>, String> {
    let Ok(mut value) = serde_json::from_str::<Value>(raw_json) else {
        // Not valid JSON at all: let the healing deserializer report it and fall back to
        // defaults, exactly as it does for any other unparseable file.
        return crate::deserialize_with_self_healing(raw_json);
    };
    if !migrate_value(&mut value) {
        return crate::deserialize_with_self_healing(raw_json);
    }
    let migrated = serde_json::to_string(&value).map_err(|e| e.to_string())?;
    let mut outcome = crate::deserialize_with_self_healing(&migrated)?;
    outcome.repaired_paths.insert(0, "<parameter units>".to_string());
    outcome.needs_rewrite = true;
    Ok(outcome)
}

/// `spring` 1–500 → `spring / 1000`; `damping` 0–99 (% lost) → `1 - damping / 100`.
fn convert_to_natural_units(config: &mut AppConfig) {
    let t = &mut config.trail;
    t.spring /= 1000.0;
    t.head_spring /= 1000.0;
    t.damping = 1.0 - t.damping / 100.0;
    t.head_damping = 1.0 - t.head_damping / 100.0;

    let h = &mut config.head;
    h.squish_intensity /= 100.0;
    h.squish_smoothing /= 100.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_current_config_is_left_alone() {
        let mut cfg = AppConfig::default();
        let before = cfg.clone();
        assert!(!migrate(&mut cfg));
        assert_eq!(cfg, before);
    }

    #[test]
    fn legacy_spring_and_percent_values_become_natural_fractions() {
        let mut cfg = AppConfig {
            schema_version: 1,
            trail: crate::config::TrailConfig {
                spring: 50.0,
                head_spring: 40.0,
                damping: 30.0,
                head_damping: 20.0,
                ..crate::config::TrailConfig::default()
            },
            head: crate::config::HeadConfig {
                squish_intensity: 3.0,
                squish_smoothing: 50.0,
                ..crate::config::HeadConfig::default()
            },
            ..AppConfig::default()
        };

        assert!(migrate(&mut cfg));
        assert_eq!(cfg.trail.spring, 0.05);
        assert_eq!(cfg.trail.head_spring, 0.04);
        assert_eq!(cfg.trail.damping, 0.7);
        assert_eq!(cfg.trail.head_damping, 0.8);
        assert_eq!(cfg.head.squish_intensity, 0.03);
        assert_eq!(cfg.head.squish_smoothing, 0.5);
        assert_eq!(cfg.schema_version, CURRENT_SCHEMA_VERSION);
        // Converted values must already be inside the new bounds.
        assert!(cfg.sanitize().is_empty());
    }

    #[test]
    fn migration_is_idempotent() {
        let mut cfg = AppConfig {
            schema_version: 1,
            trail: crate::config::TrailConfig {
                spring: 50.0,
                ..crate::config::TrailConfig::default()
            },
            ..AppConfig::default()
        };
        assert!(migrate(&mut cfg));
        let after = cfg.clone();
        assert!(!migrate(&mut cfg));
        assert_eq!(cfg, after);
    }

    #[test]
    fn a_file_without_a_schema_version_is_converted_not_skipped() {
        // The real on-disk shape: an old file has no `schema_version` key at all. Reading the
        // version out of the typed struct would pick up the *current* default and skip it.
        let raw = r#"{
            "enabled": true,
            "trail": { "spring": 50.0, "damping": 30.0, "head_spring": 40.0, "head_damping": 20.0 },
            "head": { "squish_intensity": 3.0, "squish_smoothing": 50.0 }
        }"#;
        let mut value: serde_json::Value = serde_json::from_str(raw).unwrap();
        assert!(migrate_value(&mut value));
        assert_eq!(value["trail"]["spring"], serde_json::json!(0.05));
        assert_eq!(value["trail"]["damping"], serde_json::json!(0.7));
        assert_eq!(value["trail"]["head_spring"], serde_json::json!(0.04));
        assert_eq!(value["trail"]["head_damping"], serde_json::json!(0.8));
        assert_eq!(value["head"]["squish_intensity"], serde_json::json!(0.03));
        assert_eq!(value["schema_version"], serde_json::json!(CURRENT_SCHEMA_VERSION));
        // A converted document is a no-op the second time.
        assert!(!migrate_value(&mut value));
    }

    #[test]
    fn a_current_file_is_left_untouched() {
        let value = serde_json::to_value(AppConfig::default()).unwrap();
        let mut parsed = value;
        assert!(!migrate_value(&mut parsed));
    }
}
