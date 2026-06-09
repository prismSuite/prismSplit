// src/preview.rs
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use rodio::{OutputStream, Sink};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StemPreview {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub peaks: Vec<f32>,
    pub is_playing: bool,
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

#[derive(Default)]
pub struct PlaybackController {
    _stream: Option<OutputStream>,
    sink: Option<Sink>,
}

impl PlaybackController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn play(&mut self, file_path: &str) -> Result<(), anyhow::Error> {
        self.stop();

        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        
        let file = File::open(file_path)?;
        let decoder = rodio::Decoder::new(BufReader::new(file))?;
        
        sink.append(decoder);
        sink.play();
        
        self._stream = Some(stream);
        self.sink = Some(sink);
        
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self._stream = None;
    }

    pub fn is_playing(&self) -> bool {
        self.sink.as_ref().map(|s| !s.empty()).unwrap_or(false)
    }
}
