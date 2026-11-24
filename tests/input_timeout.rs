#![cfg(unix)]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

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
fn timeout_is_passed_as_microseconds_before_input() {
    let dir = std::env::temp_dir().join(format!("ffprobe-timeout-argv-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (bin, argv_file) = fake_bin(&dir);
    let config = easy_ffprobe::Config::new()
        .ffprobe_bin(&bin)
        .timeout(Duration::from_millis(2500));
    let _ = easy_ffprobe::ffprobe_config(config, "http://media.example/a.mp4");
    let argv: Vec<String> = std::fs::read_to_string(&argv_file)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    let pos = argv
        .iter()
        .position(|a| a == "-timeout")
        .expect("-timeout passed");
    assert_eq!(argv[pos + 1], "2500000");
    assert_eq!(argv.last().unwrap(), "http://media.example/a.mp4");
    assert!(
        pos + 1 < argv.len() - 1,
        "timeout value must precede the input"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Real ffprobe against a server that sends a partial body and then stalls.
/// With `-timeout` set, ffprobe must give up in a few seconds instead of hanging.
#[test]
fn real_ffprobe_gives_up_on_a_stalled_input() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        if let Ok((mut sock, _)) = listener.accept() {
            let mut buf = [0u8; 4096];
            let _ = sock.read(&mut buf);
            let _ = sock.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 100000000\r\nContent-Type: video/mp4\r\n\r\n",
            );
            let _ = sock.write_all(&[0u8; 4096]);
            // Hold the connection open with no more data.
            std::thread::sleep(Duration::from_secs(15));
        }
    });

    let config = easy_ffprobe::Config::new().timeout(Duration::from_secs(2));
    let started = Instant::now();
    let result = easy_ffprobe::ffprobe_config(config, format!("http://{addr}/stall.mp4"));
    let elapsed = started.elapsed();

    assert!(result.is_err(), "stalled probe must fail");
    assert!(
        elapsed < Duration::from_secs(10),
        "probe took {elapsed:?}, -timeout did not apply"
    );
    drop(server);
}
