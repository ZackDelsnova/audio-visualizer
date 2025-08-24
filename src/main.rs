use macroquad::prelude::*;
use rodio::{
    Decoder,
    OutputStream,
    Sink,
    Source
};
use std::{
    env,
    fs::File,
    io::BufReader,
    sync::{
        Arc,
        Mutex
    },
    vec
};

struct RingBuffer {
    data: Vec<f32>,
    capacity: usize,
    index: usize,
}

impl RingBuffer {
    fn new(capacity: usize) -> Self {
        RingBuffer { 
            data: vec![0.0; capacity],
            capacity,
            index: 0,
        }
    }

    fn push(&mut self, value: f32) {
        self.data[self.index] = value;
        self.index = (self.index + 1) % self.capacity;
    }

    fn as_vec(&self) -> Vec<f32> {
        let mut v = Vec::with_capacity(self.capacity);
        v.extend_from_slice(&self.data[self.index..]);
        v.extend_from_slice(&self.data[..self.index]);
        v
    }
}

struct VisualizerSource<S> {
    inner: S,
    buffer: Arc<Mutex<RingBuffer>>,
}

impl<S> Iterator for VisualizerSource<S>
where
    S: Iterator<Item = i16>,
{
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sample) = self.inner.next() {
            let normalized = sample as f32 / i16::MAX as f32;
            self.buffer.lock().unwrap().push(normalized);
            Some(sample)
        } else {
            None
        }
    }
}

impl<S> Source for VisualizerSource<S>
where
    S: Source<Item = i16>,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }
    fn channels(&self) -> u16 {
        self.inner.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.inner.total_duration()
    }
}

#[macroquad::main("audio visualizer")]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("usage: cargo run -- <audiofile> (in double quotes)");
        return;
    }

    let bg_color = if args.len() > 2 { parse_hex(&args[2]) } else { BLACK };
    let bar_color = if args.len() > 3 { parse_hex(&args[3]) } else { WHITE };

    let file_path = &args[1];
    let buffer = Arc::new(Mutex::new(RingBuffer::new(2048)));

    let (_stream, handle) = OutputStream::try_default().unwrap();

    let sink = Sink::try_new(&handle).unwrap();

    let file = File::open(file_path).unwrap();
    let decoder = Decoder::new(BufReader::new(file)).unwrap();

    let vis_source = VisualizerSource {
        inner: decoder,
        buffer: buffer.clone(),
    };

    sink.append(vis_source);

    let num_bars = 64;
    let smoothing = 0.2;
    let mut last_bars = vec![0.0; num_bars];

    while !sink.empty() {
        clear_background(bg_color);

        let samples = buffer.lock().unwrap().as_vec();

        let chunk_size = samples.len() / num_bars.max(1);

        for (i, chunk) in samples.chunks(chunk_size).take(num_bars).enumerate() {
            let avg_amp = chunk.iter().map(|v| v.abs()).sum::<f32>() / chunk.len() as f32;

            let bar_height = avg_amp * screen_height() * 0.8;

            last_bars[i] = last_bars[i] * (1.0 - smoothing) + bar_height * smoothing; // smoothing 

            let bar_width = screen_width() / num_bars as f32;

            let x = i as f32 * bar_width;
            let y = screen_height() - last_bars[i];

            draw_rectangle(x, y, bar_width * 0.9, last_bars[i], bar_color);
        }

        next_frame().await;
    }
}

fn parse_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    let value = u32::from_str_radix(hex, 16).expect("invalid hex color");

    let value = match hex.len() {
        6 => (value << 8) | 0xff,
        8 => value,
        _ => panic!("hex color in rrggbbaa or rrggbb"),
    };
    
    Color::from_hex(value)
}