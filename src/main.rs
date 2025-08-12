use sstv::{SSTVEncoder, create_wav_file, modes::SSTVMode};
use std::env::args;

fn main() {
    let args: Vec<String> = args().collect();

    if args.len() <= 1 {
        sstv::help();
    }

    let image_path = args[1].as_str();
    if !std::path::Path::new(image_path).exists() {
        panic!("File not found!");
    }

    let image_destination = if args.len() > 2 {
        args[2].as_str()
    } else {
        let unix_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        &format!("SSTV_Signal{unix_timestamp}.wav")
    };

    let sstv_mode: Option<SSTVMode> = if args.len() > 3 {
        match args[3].as_str() {
            "MartinM1" => Some(SSTVMode::MartinM1),
            "MartinM2" => Some(SSTVMode::MartinM2),
            "MartinM3" => Some(SSTVMode::MartinM3),
            "MartinM4" => Some(SSTVMode::MartinM4),

            "ScottieS1" => Some(SSTVMode::ScottieS1),
            "ScottieS2" => Some(SSTVMode::ScottieS2),
            "ScottieS3" => Some(SSTVMode::ScottieS3),
            "ScottieS4" => Some(SSTVMode::ScottieS4),
            "ScottieDX" => Some(SSTVMode::ScottieDX),

            "RobotBW8" => Some(SSTVMode::BWSC1s8),

            "WraaseSC2-180" => Some(SSTVMode::WraaseSC2s180),
            _ => None,
        }
    } else {
        None
    };

    let sstv_context: SSTVEncoder = SSTVEncoder::new(image_path, sstv_mode).unwrap();
    let data = sstv_context.generate_audio_data().unwrap();
    std::fs::write(image_destination, create_wav_file(&data)).unwrap();
}
