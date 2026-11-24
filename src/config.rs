use crate::{error::FfProbeError, ffprobe::FfProbe, ffprobe_config, IntoFfprobeArg};

/// ffprobe configuration.
///
/// Use [`Config::new`] for constructing a new config.
#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) count_frames: bool,
    pub(crate) ffprobe_bin: std::path::PathBuf,
    pub(crate) http_proxy: Option<String>,
    pub(crate) timeout: Option<std::time::Duration>,
    pub(crate) probesize: Option<u64>,
    pub(crate) analyzeduration: Option<std::time::Duration>,
    pub(crate) protocol_whitelist: Option<String>,
}

impl Config {
    /// Construct a new Config.
    pub fn new() -> Config {
        Config {
            count_frames: false,
            ffprobe_bin: "ffprobe".into(),
            http_proxy: None,
            timeout: None,
            probesize: None,
            analyzeduration: None,
            protocol_whitelist: None,
        }
    }

    /// Enable the -count_frames setting.
    /// Will fully decode the file and count the frames.
    /// Frame count will be available in [`Stream::nb_read_frames`].
    pub fn count_frames(mut self, count_frames: bool) -> Self {
        self.count_frames = count_frames;
        self
    }

    /// Specify which binary name (e.g. `"ffprobe-6"`) or path (e.g. `"/opt/bin/ffprobe"`) to use
    /// for executing `ffprobe`.
    pub fn ffprobe_bin(mut self, ffprobe_bin: impl AsRef<std::path::Path>) -> Self {
        self.ffprobe_bin = ffprobe_bin.as_ref().to_path_buf();
        self
    }

    /// Route the input's HTTP(S) requests through this proxy, as ffmpeg's
    /// `-http_proxy` option (for example `"http://127.0.0.1:3128"`).
    pub fn http_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.http_proxy = Some(proxy.into());
        self
    }

    /// Limit network I/O for the input, as ffmpeg's `-timeout` option.
    /// The value is passed in microseconds, so sub-microsecond precision is dropped.
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Cap the bytes ffprobe reads while probing the container, as ffmpeg's
    /// `-probesize`. Given in bytes (ffmpeg accepts a plain integer).
    pub fn probesize(mut self, bytes: u64) -> Self {
        self.probesize = Some(bytes);
        self
    }

    /// Bound the container analysis time, as ffmpeg's `-analyzeduration`. The
    /// value is passed in microseconds.
    pub fn analyzeduration(mut self, duration: std::time::Duration) -> Self {
        self.analyzeduration = Some(duration);
        self
    }

    /// Restrict the protocols ffprobe may open, as ffmpeg's
    /// `-protocol_whitelist` (a comma-separated list, for example
    /// `"http,https,tls,tcp"`). Unset leaves ffmpeg's default list.
    pub fn protocol_whitelist(mut self, protocols: impl Into<String>) -> Self {
        self.protocol_whitelist = Some(protocols.into());
        self
    }

    /// Run ffprobe with the config produced by this builder.
    pub fn run<'a, T: IntoFfprobeArg<'a>>(self, path: T) -> Result<FfProbe, FfProbeError> {
        ffprobe_config(self, path)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
