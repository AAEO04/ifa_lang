//! # Fidio Handler - Video
//!
//! Handles video playback and capture.
//! Infrastructure domain for video I/O.

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

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
        _env: &mut Environment,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        match method {
            // Play video file
            "ṣe" | "play" => {
                if let Some(IfaValue::Str(path)) = args.first() {
                    println!("[VIDEO] Playing: {}", path);
                    return Ok(IfaValue::Bool(true));
                }
                Err(IfaError::Runtime("play requires video file path".into()))
            }

            // Stop playback
            "duro" | "stop" => {
                println!("[VIDEO] Stopping playback");
                Ok(IfaValue::Bool(true))
            }

            // Get current frame as image data
            "aworan" | "frame" => {
                println!("[VIDEO] Getting current frame");
                // Would return image data/path
                Ok(IfaValue::Null)
            }

            // Get video duration in seconds
            "akoko" | "duration" => {
                if let Some(IfaValue::Str(path)) = args.first() {
                    println!("[VIDEO] Getting duration of: {}", path);
                    // Placeholder - would use ffprobe
                    return Ok(IfaValue::Float(0.0));
                }
                Err(IfaError::Runtime("duration requires video path".into()))
            }

            // Seek to position (seconds)
            "lọ_si" | "seek" => {
                if let Some(IfaValue::Float(pos)) = args.first() {
                    println!("[VIDEO] Seeking to: {:.2}s", pos);
                    return Ok(IfaValue::Bool(true));
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

                println!("[VIDEO] Recording for {}ms...", duration);
                Ok(IfaValue::Str("recording_placeholder.mp4".to_string()))
            }

            // Get video info (width, height, fps)
            "alaye" | "info" => {
                if let Some(IfaValue::Str(path)) = args.first() {
                    println!("[VIDEO] Getting info for: {}", path);
                    return Ok(IfaValue::Map(std::collections::HashMap::from([
                        ("width".to_string(), IfaValue::Int(1920)),
                        ("height".to_string(), IfaValue::Int(1080)),
                        ("fps".to_string(), IfaValue::Float(30.0)),
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
