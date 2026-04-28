//! Historical aggregation helpers.

use std::str::FromStr;

use openwebhmi_protocol::TagValue;
use thiserror::Error;

use crate::store::HistoryPoint;

/// Supported history query aggregation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aggregation {
    /// Return raw samples.
    Raw,
    /// Numeric average per bucket.
    Avg,
    /// Numeric minimum per bucket.
    Min,
    /// Numeric maximum per bucket.
    Max,
    /// Sample count per bucket.
    Count,
    /// Numeric sum per bucket.
    Sum,
    /// First sample per bucket.
    First,
    /// Last sample per bucket.
    Last,
}

/// Aggregation failure.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AggregationError {
    /// The aggregation name is unknown.
    #[error("unsupported aggregation '{0}'")]
    UnsupportedAggregation(String),
    /// The aggregation requires numeric tag values.
    #[error("aggregation requires numeric values")]
    NonNumeric,
}

impl FromStr for Aggregation {
    type Err = AggregationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "raw" => Ok(Self::Raw),
            "avg" => Ok(Self::Avg),
            "min" => Ok(Self::Min),
            "max" => Ok(Self::Max),
            "count" => Ok(Self::Count),
            "sum" => Ok(Self::Sum),
            "first" => Ok(Self::First),
            "last" => Ok(Self::Last),
            other => Err(AggregationError::UnsupportedAggregation(other.to_string())),
        }
    }
}

impl Aggregation {
    /// Stable wire spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Avg => "avg",
            Self::Min => "min",
            Self::Max => "max",
            Self::Count => "count",
            Self::Sum => "sum",
            Self::First => "first",
            Self::Last => "last",
        }
    }
}

/// Apply an aggregation to already-loaded samples.
///
/// Bucket boundaries are left-closed and right-open, except the final bucket
/// includes `t_end_ms`.
pub fn aggregate(
    points: &[HistoryPoint],
    t_start_ms: u64,
    t_end_ms: u64,
    aggregation: Aggregation,
    max_points: u32,
) -> Result<Vec<HistoryPoint>, AggregationError> {
    if aggregation == Aggregation::Raw || max_points == 0 || points.is_empty() {
        return Ok(points.iter().take(max_points as usize).cloned().collect());
    }

    let bucket_count = max_points.max(1) as usize;
    let width = ((t_end_ms.saturating_sub(t_start_ms)).max(1) as f64) / bucket_count as f64;
    let mut buckets: Vec<Vec<&HistoryPoint>> = vec![Vec::new(); bucket_count];
    for point in points {
        let offset = point.ts_ms.saturating_sub(t_start_ms) as f64;
        let index = ((offset / width).floor() as usize).min(bucket_count - 1);
        buckets[index].push(point);
    }

    buckets
        .into_iter()
        .enumerate()
        .filter(|(_, bucket)| !bucket.is_empty())
        .map(|(index, bucket)| {
            aggregate_bucket(
                &bucket,
                t_start_ms + (index as f64 * width) as u64,
                aggregation,
            )
        })
        .collect()
}

fn aggregate_bucket(
    bucket: &[&HistoryPoint],
    ts_ms: u64,
    aggregation: Aggregation,
) -> Result<HistoryPoint, AggregationError> {
    let first = *bucket.first().expect("non-empty bucket");
    let last = *bucket.last().expect("non-empty bucket");
    let value = match aggregation {
        Aggregation::Raw => first.value.clone(),
        Aggregation::First => first.value.clone(),
        Aggregation::Last => last.value.clone(),
        Aggregation::Count => TagValue::Int(bucket.len() as i64),
        Aggregation::Avg => TagValue::Real(numeric_sum(bucket)? / bucket.len() as f64),
        Aggregation::Sum => TagValue::Real(numeric_sum(bucket)?),
        Aggregation::Min => TagValue::Real(
            bucket
                .iter()
                .map(|point| numeric_value(&point.value))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .fold(f64::INFINITY, f64::min),
        ),
        Aggregation::Max => TagValue::Real(
            bucket
                .iter()
                .map(|point| numeric_value(&point.value))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .fold(f64::NEG_INFINITY, f64::max),
        ),
    };
    Ok(HistoryPoint {
        ts_ms,
        value,
        quality: last.quality,
    })
}

fn numeric_sum(bucket: &[&HistoryPoint]) -> Result<f64, AggregationError> {
    bucket
        .iter()
        .map(|point| numeric_value(&point.value))
        .try_fold(0.0, |acc, value| value.map(|value| acc + value))
}

fn numeric_value(value: &TagValue) -> Result<f64, AggregationError> {
    match value {
        TagValue::Int(value) => Ok(*value as f64),
        TagValue::Real(value) => Ok(*value),
        TagValue::Bool(_) | TagValue::String(_) => Err(AggregationError::NonNumeric),
    }
}
