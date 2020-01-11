extern crate rusoto_polly;
use rusoto_polly::{PollyClient, Polly, SynthesizeSpeechInput};
extern crate rusoto_core;
use rusoto_core::Region;
extern crate rodio;
use rodio::Source;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::write;
use structopt::StructOpt;
use std::path::{PathBuf, Path};
extern crate rand;
#[macro_use] extern crate text_io;
use rand::Rng;
extern crate termcolor;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(StructOpt)]
#[structopt(name = "spelling", about = "Practice your spelling!")]
struct Opts {
    /// File of new line separated words to practice
    #[structopt(name = "FILE")]
    file: PathBuf,

    /// Number of questions
    #[structopt(name = "QUESTIONS", default_value = "10")]
    questions: usize,
}

pub fn read_lines(path: &PathBuf) -> Vec<String> {
    let file = File::open(path).expect("File not found at path");
    let reader = BufReader::new(file);
    reader
        .lines()
        .into_iter()
        .map(|line| line.unwrap())
        .collect()
}

fn generate_problems(opts: &Opts) -> Vec<String> {
    let lines = read_lines(&opts.file);
    let mut problems = Vec::new();
    let mut rng = rand::thread_rng();
    while problems.len() < opts.questions {
        let selection = rng.gen_range(0, lines.len() - 1);
        let selection = lines.get(selection).unwrap();
        if !problems.contains(selection) {
            problems.push(String::from(selection));
        }
    }
    problems
}

fn main() {
    let opts = Opts::from_args();
    let device = rodio::default_output_device().unwrap();
    let problems = generate_problems(&opts);
    for (i, p) in problems.iter().enumerate() {
        println!("Question {} of {}", i + 1, opts.questions);
        println!("");
        problem(p, &device);
    }
}

fn generate_audio(text: &str) {
    let client = PollyClient::new(Region::UsEast1);

    let task = SynthesizeSpeechInput{
        engine: Some(String::from("standard")),
        language_code: None,
        lexicon_names: None,
        output_format: String::from("ogg_vorbis"),
        sample_rate: None,
        speech_mark_types: None,
        text: String::from(text),
        text_type: None,
        voice_id: String::from("Joanna"),
    };
    let future = client.synthesize_speech(task);
    let result = future.sync();
    if let Err(err) = result {
        panic!("error: {}", err);
    }
    if let Ok(output) = result {
        if let Some(bytes) = output.audio_stream {
            let buffer = bytes.to_vec();
            write(format!("{}.ogg", text), buffer).unwrap();
        }
    }
}

fn problem(problem: &str, device: &rodio::Device) {
    let filename = format!("{}.ogg", problem);
    if !Path::new(&filename).exists() {
        generate_audio(problem);
    }
    let file = File::open(filename).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    rodio::play_raw(&device, source.convert_samples());

    let result : String = loop {
        if let Ok(value) = try_read!() {
            break value
        }
    };

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    if result == problem {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap();
        println!("CORRECT!");
    } else {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red))).unwrap();
        println!("Nice try! Correct answer: '{}'", problem);
    }
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White))).unwrap();
    println!("");

}