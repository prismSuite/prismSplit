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
    
    // Decode samples and map to float 0.0 - 1.0
    let samples: Vec<f32> = decoder
        .map(|sample| (sample as f32) / (i16::MAX as f32))
        .map(|s| s.abs())
        .collect();

    if samples.is_empty() {
        return Ok(vec![0.0; num_points]);
    }

    let chunk_size = samples.len() / num_points;
    let mut peaks = Vec::with_capacity(num_points);

    for chunk in samples.chunks(if chunk_size > 0 { chunk_size } else { 1 }) {
        let mut max = 0.0f32;
        for &val in chunk {
            if val > max {
                max = val;
            }
        }
        peaks.push(max);
        if peaks.len() >= num_points {
            break;
        }
    }

    while peaks.len() < num_points {
        peaks.push(0.0);
    }

    Ok(peaks)
}

pub struct PlaybackController {
    _stream: Option<OutputStream>,
    sink: Option<Sink>,
}

impl PlaybackController {
    pub fn new() -> Self {
        Self {
            _stream: None,
            sink: None,
        }
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
