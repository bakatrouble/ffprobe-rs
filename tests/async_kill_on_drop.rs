#![cfg(all(unix, feature = "async"))]

use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

/// Dropping the `ffprobe_async_config` future (for example on a timeout)
/// must kill the ffprobe child instead of leaving it running. The fake binary
/// records its PID and `exec`s a long sleep, so the PID is the child itself.
/// It is found through PATH, so the test does not depend on `ffprobe_bin`.
#[tokio::test]
async fn dropping_the_future_kills_ffprobe() {
    let dir = std::env::temp_dir().join(format!("ffprobe-kill-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let pidfile = dir.join("pid");
    std::fs::write(
        dir.join("ffprobe"),
        format!(
            "#!/bin/sh\necho $$ > '{}'\nexec sleep 30\n",
            pidfile.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(dir.join("ffprobe"), std::fs::Permissions::from_mode(0o755)).unwrap();
    let path = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![dir.clone()];
    paths.extend(std::env::split_paths(&path));
    // Only this test runs in this binary, so changing PATH is safe here.
    std::env::set_var("PATH", std::env::join_paths(paths).unwrap());

    let timed_out = tokio::time::timeout(
        Duration::from_millis(500),
        easy_ffprobe::ffprobe_async("/nonexistent.mp4"),
    )
    .await;
    assert!(timed_out.is_err(), "probe finished early: {timed_out:?}");

    let pid = std::fs::read_to_string(&pidfile)
        .expect("fake ffprobe started")
        .trim()
        .to_owned();
    let stat_path = format!("/proc/{pid}/stat");

    // Wait for the child to be killed and reaped (or at least not running).
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let alive = match std::fs::read_to_string(&stat_path) {
            Ok(stat) => !stat.contains(") Z "),
            Err(_) => false,
        };
        if !alive {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "ffprobe child {pid} still running after drop"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    std::fs::remove_dir_all(&dir).unwrap();
}
