#![allow(unused_imports)]
// SPDX-License-Identifier: MPL-2.0

use std::collections::{BTreeMap, HashMap};

use super::compare::{ComparisonPlan, diff_outputs, values_match};
use super::detail::fixture_detail;
use super::fixture::{INDICATORS, load_fixture, parse_expect, parse_fixture, sanitise_slug};
use super::runner::{
    indicator_output_key, output_value_from_text, run_fixture, run_fixture_actual,
};
use super::types::DEFAULT_RUNNER_EXPECT_TOLERANCE;
use super::*;

const BARS: &str = r#"{
    "symbol": "NASDAQ:SPY",
    "timeframe": "1D",
    "source": "test",
    "bars": [
        {"timestamp": 1735689600, "open": 10.0, "high": 11.0, "low": 9.0, "close": 10.0, "volume": 100.0},
        {"timestamp": 1735776000, "open": 10.0, "high": 12.0, "low": 9.0, "close": 11.0, "volume": 110.0}
    ]
}"#;

const RANGE_BARS: &str = r#"{
    "symbol": "NASDAQ:SPY",
    "timeframe": "1D",
    "source": "test",
    "bars": [
        {"timestamp": 1735689600, "open": 10.0, "high": 11.0, "low": 9.0, "close": 10.0, "volume": 100.0},
        {"timestamp": 1735776000, "open": 10.0, "high": 12.0, "low": 9.0, "close": 11.0, "volume": 110.0},
        {"timestamp": 1735862400, "open": 11.0, "high": 13.0, "low": 10.0, "close": 12.0, "volume": 120.0}
    ]
}"#;

mod fixtures;
mod metadata;
mod output;
mod validation;
