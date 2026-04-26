//! Tag configuration and behavior model for `sim-rockwell`.

use std::collections::HashMap;
use std::f32::consts::TAU;
use std::time::{Duration, Instant};

use anyhow::Context;
use serde::Deserialize;

/// Simulator tag value.
#[derive(Clone, Debug)]
pub enum SimValue {
    /// Boolean value.
    Bool(bool),
    /// 32-bit signed integer.
    Dint(i32),
    /// 32-bit float.
    Real(f32),
    /// UTF-8 string.
    String(String),
}

/// One configured tag.
#[derive(Clone, Debug)]
pub struct SimTag {
    /// Current tag value.
    pub value: SimValue,
    behavior: Behavior,
}

/// Shared tag map.
pub type TagMap = HashMap<String, SimTag>;

#[derive(Clone, Debug)]
enum Behavior {
    Static,
    Latch,
    Counter {
        start: i32,
        step: i32,
        every: Duration,
    },
    Sine {
        period: Duration,
        amplitude: f32,
        offset: f32,
    },
    Toggle {
        every: Duration,
    },
}

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default)]
    tag: Vec<TagConfig>,
}

#[derive(Debug, Deserialize)]
struct TagConfig {
    name: String,
    #[serde(rename = "type")]
    ty: String,
    behavior: BehaviorConfig,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum BehaviorConfig {
    Static {
        #[serde(default)]
        value: Option<toml::Value>,
    },
    Latch {
        #[serde(default)]
        value: Option<toml::Value>,
    },
    Counter {
        #[serde(default)]
        start: i32,
        #[serde(default = "one")]
        step: i32,
        #[serde(default = "one_hundred")]
        every_ms: u64,
    },
    Sine {
        period_s: u64,
        amplitude: f32,
        offset: f32,
    },
    Toggle {
        #[serde(default = "one_thousand")]
        every_ms: u64,
    },
}

/// Build built-in tags and merge tags from an optional config file.
pub fn load_tags(config_path: Option<&std::path::Path>) -> anyhow::Result<TagMap> {
    let mut tags = builtin_tags();
    if let Some(path) = config_path.filter(|path| path.exists()) {
        let config = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let config: Config = toml::from_str(&config)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        for tag in config.tag {
            tags.insert(tag.name.clone(), tag.try_into()?);
        }
    }
    Ok(tags)
}

/// Update all dynamic tag values.
pub fn update_dynamic_tags(tags: &mut TagMap, started_at: Instant) {
    let elapsed = started_at.elapsed();
    for tag in tags.values_mut() {
        match tag.behavior {
            Behavior::Static | Behavior::Latch => {}
            Behavior::Counter { start, step, every } => {
                let ticks = elapsed.as_millis() / every.as_millis().max(1);
                tag.value =
                    SimValue::Dint(start.saturating_add((ticks as i32).saturating_mul(step)));
            }
            Behavior::Sine {
                period,
                amplitude,
                offset,
            } => {
                let phase = elapsed.as_secs_f32() / period.as_secs_f32().max(0.001);
                tag.value = SimValue::Real(offset + amplitude * (TAU * phase).sin());
            }
            Behavior::Toggle { every } => {
                let ticks = elapsed.as_millis() / every.as_millis().max(1);
                tag.value = SimValue::Bool(ticks % 2 == 0);
            }
        }
    }
}

/// Latch a written value into a tag.
pub fn write_tag(tags: &mut TagMap, name: String, value: SimValue) {
    tags.entry(name)
        .and_modify(|tag| tag.value = value.clone())
        .or_insert(SimTag {
            value,
            behavior: Behavior::Latch,
        });
}

fn builtin_tags() -> TagMap {
    HashMap::from([
        (
            "Counter".to_string(),
            SimTag {
                value: SimValue::Dint(0),
                behavior: Behavior::Counter {
                    start: 0,
                    step: 1,
                    every: Duration::from_millis(100),
                },
            },
        ),
        (
            "Setpoint".to_string(),
            SimTag {
                value: SimValue::Real(0.0),
                behavior: Behavior::Latch,
            },
        ),
        (
            "Pressure".to_string(),
            SimTag {
                value: SimValue::Real(250.0),
                behavior: Behavior::Sine {
                    period: Duration::from_secs(60),
                    amplitude: 100.0,
                    offset: 250.0,
                },
            },
        ),
        (
            "Heartbeat".to_string(),
            SimTag {
                value: SimValue::Bool(true),
                behavior: Behavior::Toggle {
                    every: Duration::from_secs(1),
                },
            },
        ),
    ])
}

impl TryFrom<TagConfig> for SimTag {
    type Error = anyhow::Error;

    fn try_from(config: TagConfig) -> Result<Self, Self::Error> {
        let value = value_from_config(&config.ty, &config.behavior)?;
        let behavior = match config.behavior {
            BehaviorConfig::Static { .. } => Behavior::Static,
            BehaviorConfig::Latch { .. } => Behavior::Latch,
            BehaviorConfig::Counter {
                start,
                step,
                every_ms,
            } => Behavior::Counter {
                start,
                step,
                every: Duration::from_millis(every_ms),
            },
            BehaviorConfig::Sine {
                period_s,
                amplitude,
                offset,
            } => Behavior::Sine {
                period: Duration::from_secs(period_s),
                amplitude,
                offset,
            },
            BehaviorConfig::Toggle { every_ms } => Behavior::Toggle {
                every: Duration::from_millis(every_ms),
            },
        };
        Ok(Self { value, behavior })
    }
}

fn value_from_config(ty: &str, behavior: &BehaviorConfig) -> anyhow::Result<SimValue> {
    let raw = match behavior {
        BehaviorConfig::Static { value } | BehaviorConfig::Latch { value } => value.as_ref(),
        _ => None,
    };
    match ty {
        "BOOL" => Ok(SimValue::Bool(
            raw.and_then(toml::Value::as_bool).unwrap_or(false),
        )),
        "DINT" => Ok(SimValue::Dint(
            raw.and_then(toml::Value::as_integer).unwrap_or(0) as i32,
        )),
        "REAL" => Ok(SimValue::Real(
            raw.and_then(toml::Value::as_float).unwrap_or(0.0) as f32,
        )),
        "STRING" => Ok(SimValue::String(
            raw.and_then(toml::Value::as_str).unwrap_or("").to_string(),
        )),
        other => anyhow::bail!("unsupported tag type {other}; expected BOOL, DINT, REAL, STRING"),
    }
}

fn one() -> i32 {
    1
}

fn one_hundred() -> u64 {
    100
}

fn one_thousand() -> u64 {
    1_000
}
