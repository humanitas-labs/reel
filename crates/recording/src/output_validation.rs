use std::{path::Path, time::Duration};

use cap_enc_ffmpeg::remux::get_media_duration;
use tracing::debug;

/// Tolerated difference between the display track's container duration and the
/// media span the recorder persisted, before the recording is flagged as
/// having suspicious sync. Generous enough for muxer rounding and trailing
/// keyframe padding; far below the hundreds of milliseconds a real timestamp
/// bug produces.
const SYNC_SPAN_TOLERANCE_SECS: f64 = 0.5;
const SYNC_SPAN_TOLERANCE_RATIO: f64 = 0.03;

/// Cross-checks a finalized display track against the media duration the
/// recorder derived from the capture timestamps it actually muxed.
///
/// The two are produced independently: the expected duration comes from the
/// pipeline's timestamp span, the container duration from what the encoder
/// and muxer wrote. A container SHORTER than the span means timestamps were
/// mangled between the pipeline and the file — the class of bug that
/// silently desyncs audio/video. A LONGER container is legitimate for VFR
/// content: muxers extend the final frame through any trailing static-screen
/// hold (AVFoundation ends the session at the wall-clock stop time), so that
/// direction is only noted at debug level. Non-fatal: logs a structured
/// warning and returns the mismatch so callers can surface it.
pub fn check_display_sync_span(display_path: &Path, expected: Duration) -> Option<f64> {
    let container = get_media_duration(display_path)?;
    let shortfall = expected.as_secs_f64() - container.as_secs_f64();
    let tolerance =
        (expected.as_secs_f64() * SYNC_SPAN_TOLERANCE_RATIO).max(SYNC_SPAN_TOLERANCE_SECS);
    if shortfall > tolerance {
        tracing::error!(
            path = %display_path.display(),
            container_secs = container.as_secs_f64(),
            expected_secs = expected.as_secs_f64(),
            delta_secs = shortfall,
            "SYNC INVARIANT VIOLATION: display track duration is shorter than \
             the muxed timestamp span; this recording may have desynced audio/video"
        );
        Some(shortfall)
    } else {
        debug!(
            path = %display_path.display(),
            container_secs = container.as_secs_f64(),
            expected_secs = expected.as_secs_f64(),
            "display track duration consistent with muxed timestamp span"
        );
        None
    }
}
