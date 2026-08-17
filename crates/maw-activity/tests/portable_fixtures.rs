#![allow(clippy::float_cmp)]

use maw_activity::{classify_snapshots, is_stuck_snapshot, normalize_snapshot, ActivitySample};
use serde_json::Value;

const ACTIVITY_CLASSIFICATION_FIXTURES_JSON: &str =
    include_str!("fixtures/activity-classification.fixtures.json");

#[cfg(feature = "fixtures")]
#[test]
fn fixtures_feature_exports_the_same_corpus() {
    assert_eq!(
        maw_activity::ACTIVITY_CLASSIFICATION_FIXTURES_JSON,
        ACTIVITY_CLASSIFICATION_FIXTURES_JSON
    );
}

#[test]
fn activity_fixtures_match_maw_js() {
    let root: Value =
        serde_json::from_str(ACTIVITY_CLASSIFICATION_FIXTURES_JSON).expect("activity fixtures");
    for case in root["normalize"].as_array().expect("normalize cases") {
        assert_eq!(
            normalize_snapshot(case["input"].as_str().expect("input")),
            case["expected"].as_str().expect("expected"),
            "{}",
            case["name"]
        );
    }
    for case in root["stuck"].as_array().expect("stuck cases") {
        assert_eq!(
            is_stuck_snapshot(case["input"].as_str().expect("input")),
            case["expected"].as_bool().expect("expected"),
            "{}",
            case["name"]
        );
    }
    for case in root["classify"].as_array().expect("classify cases") {
        let samples = case["samples"]
            .as_array()
            .expect("samples")
            .iter()
            .map(|sample| ActivitySample {
                text: sample[0].as_str().expect("sample text").to_owned(),
                at_ms: sample[1].as_u64().expect("sample timestamp"),
            })
            .collect::<Vec<_>>();
        let result = classify_snapshots(
            case["pane"].as_str().expect("pane"),
            &samples,
            case["window_ms"].as_u64().expect("window"),
        );
        let expected = &case["expected"];
        assert_eq!(
            (
                result.pane.as_str(),
                result.state.as_str(),
                result.confidence.as_str(),
                u64::from(result.samples),
                u64::from(result.diff_samples),
                result.last_change_ago_seconds,
                result.sample_window_seconds
            ),
            (
                case["pane"].as_str().expect("pane"),
                expected["state"].as_str().expect("state"),
                expected["confidence"].as_str().expect("confidence"),
                expected["samples"].as_u64().expect("count"),
                expected["diff_samples"].as_u64().expect("diff count"),
                expected["last_change_ago_seconds"].as_f64().expect("age"),
                expected["sample_window_seconds"]
                    .as_f64()
                    .expect("window seconds")
            ),
            "{}",
            case["name"]
        );
    }
}
