// src/preview.rs
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use rodio::{OutputStream, Sink};

use egui::{Rect, Shape};

pub type WaveformCache = std::sync::Arc<std::sync::Mutex<Option<(Rect, bool, Vec<Shape>)>>>;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StemPreview {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub peaks: Vec<f32>,
    pub is_playing: bool,
    #[serde(skip)]
    pub cached_shapes: WaveformCache,
}

pub fn analyze_audio_peaks<P: AsRef<Path>>(path: P, num_points: usize) -> Result<Vec<f32>, anyhow::Error> {
    let file = File::open(path)?;
    let decoder = rodio::Decoder::new(BufReader::new(file))?;

    let mut peaks = Vec::with_capacity(num_points * 2);
    let mut current_max = 0.0f32;
    let mut count = 0;
    let mut chunk_size = 1;

    for sample in decoder {
        let val = (sample as f32 / i16::MAX as f32).abs();
        if val > current_max {
            current_max = val;
        }
        count += 1;
        if count >= chunk_size {
            peaks.push(current_max);
            current_max = 0.0;
            count = 0;

            if peaks.len() >= num_points * 2 {
                for i in 0..num_points {
                    peaks[i] = peaks[2 * i].max(peaks[2 * i + 1]);
                }
                peaks.truncate(num_points);
                chunk_size *= 2;
            }
        }
    }

    if count > 0 {
        peaks.push(current_max);
    }

    while peaks.len() > num_points {
        let new_len = peaks.len().div_ceil(2);
        for i in 0..new_len {
            let idx1 = 2 * i;
            let idx2 = 2 * i + 1;
            if idx2 < peaks.len() {
                peaks[i] = peaks[idx1].max(peaks[idx2]);
            } else {
                peaks[i] = peaks[idx1];
            }
        }
        peaks.truncate(new_len);
    }

    while peaks.len() < num_points {
        peaks.push(0.0);
    }

    Ok(peaks)
}

pub struct PlaybackController {
    stream: Option<(OutputStream, rodio::OutputStreamHandle)>,
    sink: Option<Sink>,
}

impl Default for PlaybackController {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaybackController {
    pub fn new() -> Self {
        let stream = OutputStream::try_default().ok();
        let sink = stream.as_ref().and_then(|(_, handle)| Sink::try_new(handle).ok());
        Self {
            stream,
            sink,
        }
    }

    pub fn play(&mut self, file_path: &str) -> Result<(), anyhow::Error> {
        self.stop();

        if self.stream.is_none() || self.sink.is_none() {
            match OutputStream::try_default() {
                Ok((stream, handle)) => {
                    match Sink::try_new(&handle) {
                        Ok(sink) => {
                            self.stream = Some((stream, handle));
                            self.sink = Some(sink);
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!("Failed to create audio sink: {:?}", e));
                        }
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to open default audio device: {:?}", e));
                }
            }
        }

        let file = File::open(file_path)?;
        let decoder = rodio::Decoder::new(BufReader::new(file))?;

        if let Some(ref sink) = self.sink {
            sink.append(decoder);
            sink.play();
        }

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(ref sink) = self.sink {
            sink.stop();
        }
        if let Some((_, ref handle)) = self.stream {
            match Sink::try_new(handle) {
                Ok(new_sink) => {
                    self.sink = Some(new_sink);
                }
                Err(e) => {
                    eprintln!("WARNING: Failed to recreate audio Sink on stop: {:?}", e);
                    self.sink = None;
                }
            }
        }
    }

    pub fn is_playing(&self) -> bool {
        self.sink.as_ref().map(|s| !s.empty()).unwrap_or(false)
    }
}
