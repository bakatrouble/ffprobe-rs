#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

/// Writes a fake ffprobe that records its argv (one per line) and exits 3.
fn fake_bin(dir: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let bin = dir.join("fake-ffprobe");
    let argv = dir.join("argv");
    std::fs::write(
        &bin,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nexit 3\n",
            argv.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
    (bin, argv)
}

#[test]
fn analyzeduration_is_passed_in_microseconds_before_input() {
    let dir = std::env::temp_dir().join(format!("ffprobe-analyze-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (bin, argv_file) = fake_bin(&dir);
    let config = easy_ffprobe::Config::new()
        .ffprobe_bin(&bin)
        .analyzeduration(Duration::from_secs(5));
    let _ = easy_ffprobe::ffprobe_config(config, "http://media.example/a.mp4");
    let argv: Vec<String> = std::fs::read_to_string(&argv_file)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    let pos = argv
        .iter()
        .position(|a| a == "-analyzeduration")
        .expect("-analyzeduration passed");
    assert_eq!(argv[pos + 1], "5000000");
    assert_eq!(argv.last().unwrap(), "http://media.example/a.mp4");
    assert!(pos + 1 < argv.len() - 1, "value must precede the input");
    std::fs::remove_dir_all(&dir).unwrap();
}
