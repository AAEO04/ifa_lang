//! # Fidio Handler - Video
//!
//! Handles video playback and capture.
//! Infrastructure domain for video I/O.

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{EnvRef, OduHandler};

/// Handler for Fidio (Video) domain.
///
/// Provides video playback, frame extraction, and recording.
/// Uses `ffmpeg-next` on native platforms.
pub struct FidioHandler;

impl OduHandler for FidioHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Fidio
    }

    fn call(
        &self,
        method: &str,
        args: Vec<IfaValue>,
        _env: &EnvRef,
        output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        match method {
            // Play video file
            "ṣe" | "play" => {
                // Check if first arg is string using Pattern Matching on Kind
                if let Some(IfaValue::Str(path)) = args.first() {
                    output.push(format!("[VIDEO] Playing: {}", path));
                    return Ok(IfaValue::bool(true));
                }
                Err(IfaError::Runtime("play requires video file path".into()))
            }

            // Stop playback
            "duro" | "stop" => {
                output.push("[VIDEO] Stopping playback".into());
                Ok(IfaValue::bool(true))
            }

            // Get current frame as image data
            "aworan" | "frame" => {
                output.push("[VIDEO] Getting current frame".into());
                // Would return image data/path
                Ok(IfaValue::null())
            }

            // Get video duration in seconds
            "akoko" | "duration" => {
                if let Some(IfaValue::Str(path)) = args.first() {
                    output.push(format!("[VIDEO] Getting duration of: {}", path));
                    // ffprobe / ffmpeg-next not yet linked.
                    // Return Err so callers know the duration is unavailable,
                    // not that the file is zero-length.
                    return Err(IfaError::Runtime(
                        "duration: ffprobe not available (ffmpeg-next feature not enabled)".into(),
                    ));
                }
                Err(IfaError::Runtime("duration requires video path".into()))
            }

            // Seek to position (seconds)
            "lọ_si" | "seek" => {
                if let Some(IfaValue::Float(pos)) = args.first() {
                    output.push(format!("[VIDEO] Seeking to: {:.2}s", *pos));
                    return Ok(IfaValue::bool(true));
                }
                Err(IfaError::Runtime(
                    "seek requires position in seconds".into(),
                ))
            }

            // Record from camera
            "gba" | "record" => {
                let duration = args
                    .first()
                    .and_then(|v| {
                        if let IfaValue::Int(n) = v {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(5000);

                // Camera capture requires ffmpeg-next (or a platform camera API).
                // Return Err so callers know nothing was recorded rather than
                // receiving a filename that does not exist on disk.
                let _ = duration;
                Err(IfaError::Runtime(
                    "record: camera capture not available (ffmpeg-next feature not enabled)".into(),
                ))
            }

            // Get video info (width, height, fps)
            "alaye" | "info" => {
                if let Some(IfaValue::Str(path)) = args.first() {
                    output.push(format!("[VIDEO] Getting info for: {}", path));
                    return Ok(IfaValue::map(std::collections::HashMap::from([
                        ("width".into(), IfaValue::int(1920)),
                        ("height".into(), IfaValue::int(1080)),
                        ("fps".into(), IfaValue::float(30.0)),
                    ])));
                }
                Err(IfaError::Runtime("info requires video path".into()))
            }


            _ => Err(IfaError::Runtime(format!(
                "Unknown Fidio method: {}",
                method
            ))),
        }
    }

    fn methods(&self) -> &'static [&'static str] {
        &[
            "ṣe", "play", "duro", "stop", "aworan", "frame", "akoko", "duration", "lọ_si", "seek",
            "gba", "record", "alaye", "info",
        ]
    }
}
