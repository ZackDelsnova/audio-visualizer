use rodio::{Decoder, Sink, OutputStream};
use std::env;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("usage: cargo run -- <audiofile> (in double quotes)");
        return;
    }

    let file_path = &args[1];
     
    let (_stream, handle) = OutputStream::try_default().expect("filed to open");

    let sink = Sink::try_new(&handle).expect("failed to make sink");

    let file = File::open(file_path).expect("filade to open file");
    let source = Decoder::new(BufReader::new(file)).expect("failed to decode audio file");

    sink.append(source);
    sink.sleep_until_end();
    
}
