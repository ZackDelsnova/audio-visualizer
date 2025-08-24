use macroquad::prelude::*;
use rodio::{Decoder, OutputStream, Sink};
use std::{env, vec};
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::sync::{Arc, Mutex};

#[macroquad::main("audio visualizer")]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("usage: cargo run -- <audiofile> (in double quotes)");
        return;
    }

    let file_path = &args[1];
    let buffer = Arc::new(Mutex::new(RingBuffer::new(800)));
    play_audio_and_fill_buffer(file_path.clone(), buffer.clone());

    loop {
        clear_background(WHITE);

        let buf_snapshot = buffer.lock().unwrap().as_vec();

        for (i, &value) in buf_snapshot.iter().enumerate() {
            let x = i as f32 * (screen_width() / buf_snapshot.len() as f32);
            let y = screen_height() / 2.0 - value * 200.0; // scale the waveform

            if i > 0 {
                let prev_x = (i as f32 - 1.0) * (screen_width() / buf_snapshot.len() as f32);
                let prev_y = screen_height() / 2.0 - buf_snapshot[i - 1] * 200.0;
                draw_line(prev_x, prev_y, x, y, 2.0, BLACK);
            }
        }

        next_frame().await;
    }
}

fn play_audio_and_fill_buffer(file_path: String, buffer: Arc<Mutex<RingBuffer>>) {
    thread::spawn(move || {
        let (_stream, handle) = OutputStream::try_default().unwrap();

        let sink = Sink::try_new(&handle).unwrap();

        let file = File::open(file_path.clone()).unwrap();
        let source = Decoder::new(BufReader::new(file)).unwrap();

        sink.append(source);

        let file = File::open(file_path).unwrap();
        let mut vis_source = Decoder::new(BufReader::new(file)).unwrap();

        for sample in vis_source.by_ref() {
            let normalized = sample as f32 / i16::MAX as f32;
            buffer.lock().unwrap().push(normalized);
            thread::sleep(std::time::Duration::from_micros(500));
        }

        sink.sleep_until_end();
    });
}

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