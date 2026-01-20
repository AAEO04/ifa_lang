//! # Ohun Handler - Audio
//!
//! Handles audio playback and recording.
//! Infrastructure domain for sound I/O.
//!
//! When `audio` feature is enabled, uses `rodio` for real audio playback.
//! Otherwise, provides stub responses.

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

// Real audio implementation when audio feature is enabled
#[cfg(feature = "audio")]
use rodio::{Decoder, OutputStream, Sink};
#[cfg(feature = "audio")]
use std::fs::File;
#[cfg(feature = "audio")]
use std::io::BufReader;

/// Handler for Ohun (Audio) domain.
/// 
/// Provides audio playback, volume control, and simple tone generation.
/// Uses `rodio` on native platforms when audio feature is enabled.
pub struct OhunHandler;

#[cfg(feature = "audio")]
fn play_audio_file(path: &str) -> IfaResult<bool> {
    let file = File::open(path)
        .map_err(|e| IfaError::Runtime(format!("Failed to open audio file: {}", e)))?;
    let reader = BufReader::new(file);
    
    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| IfaError::Runtime(format!("Failed to get audio output: {}", e)))?;
    
    let sink = Sink::try_new(&stream_handle)
        .map_err(|e| IfaError::Runtime(format!("Failed to create audio sink: {}", e)))?;
    
    let source = Decoder::new(reader)
        .map_err(|e| IfaError::Runtime(format!("Failed to decode audio: {}", e)))?;
    
    sink.append(source);
    sink.sleep_until_end();
    
    Ok(true)
}

#[cfg(not(feature = "audio"))]
fn play_audio_file(path: &str) -> IfaResult<bool> {
    Err(IfaError::Runtime(format!(
        "Audio disabled. Enable 'audio' feature to play files. Path: {}", path
    )))
}

#[cfg(feature = "audio")]
fn play_beep(freq: f32, duration_ms: u64) -> IfaResult<()> {
    use rodio::source::{SineWave, Source};
    
    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| IfaError::Runtime(format!("Failed to get audio output: {}", e)))?;
    
    let sink = Sink::try_new(&stream_handle)
        .map_err(|e| IfaError::Runtime(format!("Failed to create audio sink: {}", e)))?;
    
    let source = SineWave::new(freq)
        .take_duration(std::time::Duration::from_millis(duration_ms))
        .amplify(0.2);
    
    sink.append(source);
    sink.sleep_until_end();
    
    Ok(())
}

#[cfg(not(feature = "audio"))]
fn play_beep(_freq: f32, _duration_ms: u64) -> IfaResult<()> {
    // Fallback: ASCII bell
    print!("\x07");
    Ok(())
}

impl OduHandler for OhunHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Ohun
    }
    
    fn call(
        &self, 
        method: &str, 
        args: Vec<IfaValue>, 
        _env: &mut Environment
    ) -> IfaResult<IfaValue> {
        match method {
            // Play audio file
            "ṣe" | "play" => {
                if let Some(IfaValue::Str(path)) = args.first() {
                    let result = play_audio_file(path)?;
                    return Ok(IfaValue::Bool(result));
                }
                Err(IfaError::Runtime("play requires audio file path".into()))
            }
            
            // Stop playback (would need global state to track)
            "duro" | "stop" => {
                Ok(IfaValue::Bool(true))
            }
            
            // Pause/resume (would need global state)
            "da_duro" | "pause" => {
                Ok(IfaValue::Bool(true))
            }
            
            // Get/set volume (0.0 - 1.0)
            "ohùn" | "volume" => {
                if let Some(IfaValue::Float(vol)) = args.first() {
                    let vol = vol.clamp(0.0, 1.0);
                    return Ok(IfaValue::Float(vol));
                }
                // Return current volume if no arg
                Ok(IfaValue::Float(1.0))
            }
            
            // Simple beep/tone with real audio when feature enabled
            "gbọn" | "beep" => {
                let freq = args.first()
                    .and_then(|v| if let IfaValue::Int(n) = v { Some(*n as f32) } else { None })
                    .unwrap_or(440.0);
                let duration = args.get(1)
                    .and_then(|v| if let IfaValue::Int(n) = v { Some(*n as u64) } else { None })
                    .unwrap_or(200);
                
                play_beep(freq, duration)?;
                Ok(IfaValue::Null)
            }
            
            // Record from microphone (requires additional crate like cpal)
            "gba" | "record" => {
                Err(IfaError::Runtime(
                    "Recording requires microphone access. Use ifa-std with cpal feature.".into()
                ))
            }
            
            // Check if audio is playing
            "ṣe_ṣiṣẹ" | "is_playing" => {
                Ok(IfaValue::Bool(false))
            }
            
            // List audio devices
            #[cfg(feature = "audio")]
            "awọn_ẹrọ" | "devices" => {
                use rodio::cpal::traits::{HostTrait, DeviceTrait};
                let host = rodio::cpal::default_host();
                let devices: Vec<IfaValue> = host.output_devices()
                    .map(|devs| {
                        devs.filter_map(|d| d.name().ok())
                            .map(IfaValue::Str)
                            .collect()
                    })
                    .unwrap_or_else(|_| vec![IfaValue::Str("Default Output".to_string())]);
                Ok(IfaValue::List(devices))
            }
            
            #[cfg(not(feature = "audio"))]
            "awọn_ẹrọ" | "devices" => {
                Ok(IfaValue::List(vec![
                    IfaValue::Str("Default Output (audio feature disabled)".to_string()),
                ]))
            }
            
            _ => Err(IfaError::Runtime(format!(
                "Unknown Ohun method: {}",
                method
            ))),
        }
    }
    
    fn methods(&self) -> &'static [&'static str] {
        &["ṣe", "play", "duro", "stop", "da_duro", "pause", 
          "ohùn", "volume", "gbọn", "beep", "gba", "record",
          "ṣe_ṣiṣẹ", "is_playing", "awọn_ẹrọ", "devices"]
    }
}

