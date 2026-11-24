#![cfg(feature = "streams")]

use easy_ffprobe::{FfProbe, StreamKinds};

/// Real ffprobe 9.1 output for an H.264 video with a PNG attached picture.
const FIXTURE: &str = include_str!("fixtures/cover_ffprobe.json");

#[test]
fn full_document_parses_both_streams() {
    let probe: FfProbe = serde_json::from_str(FIXTURE).expect("fixture parses");
    assert_eq!(probe.streams.len(), 2);
    assert_eq!(probe.streams[1].disposition.attached_pic, 1);
}

#[test]
fn one_malformed_stream_does_not_fail_the_document() {
    let mut doc: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    // A non-numeric width makes the first stream's body unparsable.
    doc["streams"][0]["width"] = serde_json::Value::from("wide");
    let probe: FfProbe = serde_json::from_value(doc).expect("document still parses");
    assert_eq!(probe.streams.len(), 2);
    assert!(matches!(probe.streams[0].stream, StreamKinds::Unknown));
    assert_eq!(probe.streams[0].index, 0);
    // The attached picture after the broken stream is still found.
    assert_eq!(probe.streams[1].disposition.attached_pic, 1);
    assert_eq!(probe.streams[1].index, 1);
}

#[test]
fn non_array_streams_still_fail() {
    let mut doc: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    doc["streams"] = serde_json::Value::from("nope");
    assert!(serde_json::from_value::<FfProbe>(doc).is_err());
}
