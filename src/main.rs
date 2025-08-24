use macroquad::prelude::*;
use rodio::{Decoder, OutputStream, Sink, Source};
// use symphonia::core::sample;
use std::{env, vec};
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::sync::{Arc, Mutex};

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

    let file_path = &args[1];
    let buffer = Arc::new(Mutex::new(RingBuffer::new(2048)));
    play_audio(file_path.clone(), buffer.clone());

    let num_bars = 64;

    loop {
        clear_background(BLACK);

        let samples = buffer.lock().unwrap().as_vec();

        let chunk_size = samples.len() / num_bars.max(1);

        for (i, chunk) in samples.chunks(chunk_size).take(num_bars).enumerate() {
            let avg_amp = chunk.iter().map(|v| v.abs()).sum::<f32>() / chunk.len() as f32;

            let bar_height = avg_amp * screen_height() * 0.8;
            let bar_width = screen_width() / num_bars as f32;

            let x = i as f32 * bar_width;
            let y = screen_height() - bar_height;

            draw_rectangle(x, y, bar_width * 0.9, bar_height, BLUE);
        }

        next_frame().await;
    }
}

fn play_audio(file_path: String, buffer: Arc<Mutex<RingBuffer>>) {
    thread::spawn(move || {
        let (_stream, handle) = OutputStream::try_default().unwrap();

        let sink = Sink::try_new(&handle).unwrap();

        let file = File::open(file_path.clone()).unwrap();
        let decoder = Decoder::new(BufReader::new(file)).unwrap();

        let vis_source = VisualizerSource {
            inner: decoder,
            buffer: buffer.clone(),
        };

        sink.append(vis_source);
        sink.sleep_until_end();
    });
}
