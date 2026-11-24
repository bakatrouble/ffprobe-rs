#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;

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

fn assert_proxy_before_input(argv: &[String], proxy: &str) {
    let pos = argv
        .iter()
        .position(|a| a == "-http_proxy")
        .expect("-http_proxy passed");
    assert_eq!(argv[pos + 1], proxy);
    // The input path must stay last, after every option.
    assert_eq!(argv.last().unwrap(), "http://media.example/a.mp4");
    assert!(
        pos + 1 < argv.len() - 1,
        "proxy value must precede the input"
    );
}

#[test]
fn sync_config_passes_http_proxy_before_input() {
    let dir = std::env::temp_dir().join(format!("ffprobe-proxy-sync-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (bin, argv_file) = fake_bin(&dir);
    let config = easy_ffprobe::Config::new()
        .ffprobe_bin(&bin)
        .http_proxy("http://127.0.0.1:3128");
    let _ = easy_ffprobe::ffprobe_config(config, "http://media.example/a.mp4");
    let argv: Vec<String> = std::fs::read_to_string(&argv_file)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    assert_proxy_before_input(&argv, "http://127.0.0.1:3128");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[cfg(feature = "async")]
#[tokio::test]
async fn async_config_passes_http_proxy_before_input() {
    let dir = std::env::temp_dir().join(format!("ffprobe-proxy-async-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (bin, argv_file) = fake_bin(&dir);
    let config = easy_ffprobe::Config::new()
        .ffprobe_bin(&bin)
        .http_proxy("http://127.0.0.1:3128");
    let _ = easy_ffprobe::ffprobe_async_config(config, "http://media.example/a.mp4").await;
    let argv: Vec<String> = std::fs::read_to_string(&argv_file)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    assert_proxy_before_input(&argv, "http://127.0.0.1:3128");
    std::fs::remove_dir_all(&dir).unwrap();
}
