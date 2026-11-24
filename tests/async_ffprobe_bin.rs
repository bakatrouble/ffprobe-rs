#![cfg(all(unix, feature = "async"))]

use std::os::unix::fs::PermissionsExt;

/// `ffprobe_async_config` must run the binary named in `Config`, not `ffprobe`
/// from PATH. A fake binary records that it ran and exits non-zero.
#[tokio::test]
async fn async_config_runs_configured_ffprobe_bin() {
    let dir = std::env::temp_dir().join(format!("ffprobe-bin-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let marker = dir.join("ran");
    let bin = dir.join("fake-ffprobe");
    std::fs::write(
        &bin,
        format!("#!/bin/sh\ntouch '{}'\nexit 3\n", marker.display()),
    )
    .unwrap();
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();

    let config = easy_ffprobe::Config::new().ffprobe_bin(&bin);
    let result = easy_ffprobe::ffprobe_async_config(config, "/nonexistent.mp4").await;

    assert!(marker.exists(), "configured ffprobe_bin was not executed");
    assert!(matches!(result, Err(easy_ffprobe::error::FfProbeError::Status(_))));
    std::fs::remove_dir_all(&dir).unwrap();
}
