use image::{DynamicImage, GenericImageView, ImageReader, imageops::FilterType::Lanczos3};
use modes::{SSTVMode, get_mode_from_resolution};
use std::{
    error::Error,
    f64::consts::{PI, TAU},
};

/*
    Some code "borrowed" from BenderBlog at
    https://github.com/BenderBlog/rust-sstv/

    Very helpful!
*/

pub mod modes;

const SAMPLE_RATE: u32 = 44100;
const CHANNELS: u16 = 1;
const AUDIO_FORMAT: AudioFormat = AudioFormat::PCM;
const BLOCK_SIZE: u32 = 16;
const BITS_PER_SAMPLE: u16 = 8;

#[allow(dead_code)]
#[derive(Debug)]
enum AudioFormat {
    PCM = 1,  // Integer
    IEEE = 3, // Float
}

const fn get_wav_file_header(data: &[u8]) -> [u8; 44] {
    let file_size = data.len() + 44 - 8;

    // File size
    let a = !(file_size & 0xFFFFFF) & 0xFFFFFF;
    let a4 = (a >> 24 & 0xFF) as u8;
    let a3 = (a >> 16 & 0xFF) as u8;
    let a2 = (a >> 8 & 0xFF) as u8;
    let a1 = (a & 0xFF) as u8;

    // Block size
    let b4 = (BLOCK_SIZE >> 24 & 0xFF) as u8;
    let b3 = (BLOCK_SIZE >> 16 & 0xFF) as u8;
    let b2 = (BLOCK_SIZE >> 8 & 0xFF) as u8;
    let b1 = (BLOCK_SIZE & 0xFF) as u8;

    // Audio Format
    let c2 = (AUDIO_FORMAT as u16 >> 8 & 0xFF) as u8;
    let c1 = (AUDIO_FORMAT as u16 & 0xFF) as u8;

    // Number of channels
    let d2 = (CHANNELS >> 8 & 0xFF) as u8;
    let d1 = (CHANNELS & 0xFF) as u8;

    // Frequency
    let e4 = (SAMPLE_RATE >> 24 & 0xFF) as u8;
    let e3 = (SAMPLE_RATE >> 16 & 0xFF) as u8;
    let e2 = (SAMPLE_RATE >> 8 & 0xFF) as u8;
    let e1 = (SAMPLE_RATE & 0xFF) as u8;

    // Bits per sample
    let h2 = (BITS_PER_SAMPLE >> 8 & 0xFF) as u8;
    let h1 = (BITS_PER_SAMPLE & 0xFF) as u8;

    // Bytes per block
    let g = CHANNELS * BITS_PER_SAMPLE / 8;
    let g2 = (g >> 8 & 0xFF) as u8;
    let g1 = (g & 0xFF) as u8;

    // Bytes per second
    let f = SAMPLE_RATE * g as u32;
    let f4 = (f >> 24 & 0xFF) as u8;
    let f3 = (f >> 16 & 0xFF) as u8;
    let f2 = (f >> 8 & 0xFF) as u8;
    let f1 = (f & 0xFF) as u8;

    // Sampled data size
    let i = data.len();
    let i4 = (i >> 24 & 0xFF) as u8;
    let i3 = (i >> 16 & 0xFF) as u8;
    let i2 = (i >> 8 & 0xFF) as u8;
    let i1 = (i & 0xFF) as u8;

    [
        // Master RIFF chunk
        0x52, 0x49, 0x46, 0x46, a1, a2, a3, a4, 0x57, 0x41, 0x56, 0x45,
        // Chunk describing the data format
        0x66, 0x6D, 0x74, 0x20, b1, b2, b3, b4, c1, c2, d1, d2, e1, e2, e3, e4, f1, f2, f3, f4, g1,
        g2, h1, h2, // Chunk containing the sampled data
        0x64, 0x61, 0x74, 0x61, i1, i2, i3, i4,
    ]
}

pub fn create_wav_file(data: &[u8]) -> Vec<u8> {
    let header = get_wav_file_header(data);
    [&header, data].concat()
}

#[allow(dead_code)]
struct AudioData {
    file_size: u32,
    sample_rate: u32,
    channels: u16,
    bytes_per_second: u32,
    bytes_per_block: u16,
    audio_format: AudioFormat,
    block_size: u32,
    bits_per_sample: u16,
    sampled_data_size: u32,
}

pub struct SSTVEncoder {
    mode: SSTVMode,
    img: DynamicImage,
}

const COLOR_HIGH: u32 = 2300;
const COLOR_LOW: u32 = 1500;

impl SSTVEncoder {
    pub fn new(image_path: &str, mode_overwrite: Option<SSTVMode>) -> Result<Self, Box<dyn Error>> {
        let img_raw = ImageReader::open(image_path)?.decode()?;
        let mode: SSTVMode = if mode_overwrite.is_some() {
            mode_overwrite.unwrap()
        } else {
            get_mode_from_resolution(img_raw.width(), img_raw.height(), false)
        };

        let img = match mode {
            // Martin
            SSTVMode::MartinM1 => img_raw.resize_exact(320, 256, Lanczos3),
            SSTVMode::MartinM2 => img_raw.resize_exact(160, 256, Lanczos3),
            SSTVMode::MartinM3 => img_raw.resize_exact(320, 128, Lanczos3),
            SSTVMode::MartinM4 => img_raw.resize_exact(160, 128, Lanczos3),

            // Scottie
            SSTVMode::ScottieS1 => img_raw.resize_exact(320, 256, Lanczos3),
            SSTVMode::ScottieS2 => img_raw.resize_exact(320, 128, Lanczos3),
            SSTVMode::ScottieS3 => img_raw.resize_exact(320, 128, Lanczos3),
            SSTVMode::ScottieS4 => img_raw.resize_exact(160, 128, Lanczos3),
            SSTVMode::ScottieDX => img_raw.resize_exact(320, 256, Lanczos3),

            // B/W Mode
            SSTVMode::BWSC1s8 => img_raw.resize_exact(160, 120, Lanczos3),

            // Wraase
            SSTVMode::WraaseSC2s180 => img_raw.resize_exact(512, 256, Lanczos3),
        };

        Ok(SSTVEncoder { mode, img })
    }

    fn play_vis_code_tones(&self, vis_code: u8, sine_gen: &mut SineGen) {
        for b in 0..8 {
            if vis_code >> b & 0b1 == 0b1 {
                sine_gen.generate_samples(1100.0, 30.0); // 1
            } else {
                sine_gen.generate_samples(1300.0, 30.0); // 0
            }
        }
    }

    pub fn generate_audio_data(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let width: usize = self.img.width() as usize;
        let height: usize = self.img.height() as usize;

        let mut buffer_r: Vec<u8> = vec![0; width];
        let mut buffer_g: Vec<u8> = vec![0; width];
        let mut buffer_b: Vec<u8> = vec![0; width];

        let mut sine_gen = SineGen::new();

        let vis_code = match self.mode {
            SSTVMode::MartinM1 => 44,
            SSTVMode::MartinM2 => 40,
            SSTVMode::MartinM3 => 36,
            SSTVMode::MartinM4 => 32,

            SSTVMode::ScottieS1 => 60,
            SSTVMode::ScottieS2 => 56,
            SSTVMode::ScottieS3 => 52,
            SSTVMode::ScottieS4 => 48,
            SSTVMode::ScottieDX => 76,

            SSTVMode::BWSC1s8 => 2,

            SSTVMode::WraaseSC2s180 => 55,
        };

        let scan_line_ms = match self.mode {
            SSTVMode::MartinM1 | SSTVMode::MartinM3 => 146.432,
            SSTVMode::MartinM2 | SSTVMode::MartinM4 => 073.216,

            SSTVMode::ScottieS1 | SSTVMode::ScottieS3 => 138.240,
            SSTVMode::ScottieS2 | SSTVMode::ScottieS4 => 088.064,
            SSTVMode::ScottieDX => 345.600,

            SSTVMode::BWSC1s8 => 56.0,

            SSTVMode::WraaseSC2s180 => 235.0,
        };
        let pixel_ms = scan_line_ms / width as f64;

        let sync = match self.mode {
            SSTVMode::MartinM1 | SSTVMode::MartinM2 | SSTVMode::MartinM3 | SSTVMode::MartinM4 => {
                4.862
            }

            SSTVMode::ScottieS1
            | SSTVMode::ScottieS2
            | SSTVMode::ScottieS3
            | SSTVMode::ScottieS4
            | SSTVMode::ScottieDX => 9.0,

            SSTVMode::BWSC1s8 => 10.0,

            SSTVMode::WraaseSC2s180 => 5.5225,
        };

        let seperator_ms = match self.mode {
            SSTVMode::MartinM1 | SSTVMode::MartinM2 | SSTVMode::MartinM3 | SSTVMode::MartinM4 => {
                0.572
            }

            SSTVMode::ScottieS1
            | SSTVMode::ScottieS2
            | SSTVMode::ScottieS3
            | SSTVMode::ScottieS4
            | SSTVMode::ScottieDX => 1.5,

            SSTVMode::BWSC1s8 => 3.0,

            SSTVMode::WraaseSC2s180 => 0.5,
        };

        // VOX TONE
        sine_gen.generate_samples(1900.0, 100.0);
        sine_gen.generate_samples(1500.0, 100.0);
        sine_gen.generate_samples(1900.0, 100.0);
        sine_gen.generate_samples(1500.0, 100.0);
        sine_gen.generate_samples(2300.0, 100.0);
        sine_gen.generate_samples(1500.0, 100.0);
        sine_gen.generate_samples(2300.0, 100.0);
        sine_gen.generate_samples(1500.0, 100.0);

        // CALIBRATION HEADER
        sine_gen.generate_samples(1900.0, 300.0);
        sine_gen.generate_samples(1200.0, 10.0);
        sine_gen.generate_samples(1900.0, 300.0);
        sine_gen.generate_samples(1200.0, 30.0);

        self.play_vis_code_tones(vis_code, &mut sine_gen);

        sine_gen.generate_samples(1200.0, 30.0); // VIS stop bit

        if self.mode.is_scottie() {
            // STARTING SYNC PULS (FIRST LINE ONLY)
            sine_gen.generate_samples(1200.0, sync);

            for y in 0..height {
                for x in 0..width {
                    let pixel = self.img.get_pixel(x as u32, y as u32).0;
                    buffer_r[x] = pixel[0];
                    buffer_g[x] = pixel[1];
                    buffer_b[x] = pixel[2];
                }

                // Seperator Pulse
                sine_gen.generate_samples(1500.0, seperator_ms);

                // // GREEN SCAN
                for i in &buffer_g {
                    let freq =
                        COLOR_LOW as f64 + (COLOR_HIGH - COLOR_LOW) as f64 * *i as f64 / 255.0;
                    sine_gen.generate_samples(freq, pixel_ms);
                }

                // Seperator Pulse
                sine_gen.generate_samples(1500.0, seperator_ms);

                // // BLUE SCAN
                for i in &buffer_b {
                    let freq =
                        COLOR_LOW as f64 + (COLOR_HIGH - COLOR_LOW) as f64 * *i as f64 / 255.0;
                    sine_gen.generate_samples(freq, pixel_ms);
                }

                // Sync Pulse
                sine_gen.generate_samples(1200.0, sync);

                // Sync Porch
                sine_gen.generate_samples(1500.0, seperator_ms);

                // RED SCAN
                for i in &buffer_r {
                    let freq =
                        COLOR_LOW as f64 + (COLOR_HIGH - COLOR_LOW) as f64 * *i as f64 / 255.0;
                    sine_gen.generate_samples(freq, pixel_ms);
                }
            }
        } else if self.mode.is_martin() {
            // STARTING SYNC PULS (FIRST LINE ONLY)
            sine_gen.generate_samples(1200.0, sync);

            for y in 0..height {
                for x in 0..width {
                    let pixel = self.img.get_pixel(x as u32, y as u32).0;
                    buffer_r[x] = pixel[0];
                    buffer_g[x] = pixel[1];
                    buffer_b[x] = pixel[2];
                }

                // Sync Pulse
                sine_gen.generate_samples(1200.0, sync);

                // Sync Porch
                sine_gen.generate_samples(1500.0, seperator_ms);

                // // GREEN SCAN
                for i in &buffer_g {
                    let freq =
                        COLOR_LOW as f64 + (COLOR_HIGH - COLOR_LOW) as f64 * *i as f64 / 255.0;
                    sine_gen.generate_samples(freq, pixel_ms);
                }

                // Seperator Pulse
                sine_gen.generate_samples(1500.0, seperator_ms);

                // // BLUE SCAN
                for i in &buffer_b {
                    let freq =
                        COLOR_LOW as f64 + (COLOR_HIGH - COLOR_LOW) as f64 * *i as f64 / 255.0;
                    sine_gen.generate_samples(freq, pixel_ms);
                }

                // Seperator Pulse
                sine_gen.generate_samples(1500.0, seperator_ms);

                // RED SCAN
                for i in &buffer_r {
                    let freq =
                        COLOR_LOW as f64 + (COLOR_HIGH - COLOR_LOW) as f64 * *i as f64 / 255.0;
                    sine_gen.generate_samples(freq, pixel_ms);
                }
            }
        } else if self.mode.is_bw() {
            for y in 0..height {
                sine_gen.generate_samples(1200.0, sync);
                for x in 0..width {
                    let g_val = self.img.get_pixel(x as u32, y as u32).0[1];
                    let freq =
                        COLOR_LOW as f64 + (COLOR_HIGH - COLOR_LOW) as f64 * g_val as f64 / 255.0;
                    sine_gen.generate_samples(freq, pixel_ms);
                }
            }
        } else if self.mode.is_wraase() {
            for y in 0..height {
                for x in 0..width {
                    let pixel = self.img.get_pixel(x as u32, y as u32).0;
                    buffer_r[x] = pixel[0];
                    buffer_g[x] = pixel[1];
                    buffer_b[x] = pixel[2];
                }

                // Sync Pulse
                sine_gen.generate_samples(1200.0, sync);

                // Sync Porch
                sine_gen.generate_samples(1500.0, seperator_ms);

                // RED SCAN
                for i in &buffer_r {
                    let freq =
                        COLOR_LOW as f64 + (COLOR_HIGH - COLOR_LOW) as f64 * *i as f64 / 255.0;
                    sine_gen.generate_samples(freq, pixel_ms);
                }

                // // GREEN SCAN
                for i in &buffer_g {
                    let freq =
                        COLOR_LOW as f64 + (COLOR_HIGH - COLOR_LOW) as f64 * *i as f64 / 255.0;
                    sine_gen.generate_samples(freq, pixel_ms);
                }

                // // BLUE SCAN
                for i in &buffer_b {
                    let freq =
                        COLOR_LOW as f64 + (COLOR_HIGH - COLOR_LOW) as f64 * *i as f64 / 255.0;
                    sine_gen.generate_samples(freq, pixel_ms);
                }
            }
        }

        Ok(sine_gen.buffer)
    }
}

struct SineGen {
    older_data: f64,
    older_cos: f64,
    delta_len: f64,
    buffer: Vec<u8>,
}

impl SineGen {
    fn new() -> Self {
        SineGen {
            older_data: 0.0,
            older_cos: 0.0,
            delta_len: 0.0,
            buffer: Vec::with_capacity((SAMPLE_RATE * BITS_PER_SAMPLE as u32) as usize),
        }
    }

    fn sign(&self, val: f64) -> f64 {
        if val >= 0.0 { 1.0 } else { -1.0 }
    }

    fn generate_samples(&mut self, freq: f64, duration_ms: f64) {
        let mut samples: usize = (SAMPLE_RATE as f64 * duration_ms / 1000.0) as usize;

        // Compensating the precision if needed.
        self.delta_len += SAMPLE_RATE as f64 * duration_ms / 1000.0 - (samples as f64);
        if self.delta_len >= 1.0 {
            samples += self.delta_len as usize;
            self.delta_len -= self.delta_len.floor();
        }

        // Generate phi samples
        let phi_samples = SAMPLE_RATE as f64
            * (self.sign(self.older_cos) * self.older_data.asin()
                + (self.sign(self.older_cos) - 1.0).abs() / 2.0 * PI);

        for i in 0..samples {
            self.older_data =
                ((TAU * freq * (samples as f64) + phi_samples) / SAMPLE_RATE as f64).sin();
            self.older_cos =
                ((TAU * freq * (samples as f64) + phi_samples) / SAMPLE_RATE as f64).cos();
            self.buffer.push(
                ((u8::MAX / 2) as f64
                    * ((TAU * freq * i as f64 + phi_samples) / SAMPLE_RATE as f64).sin()
                    + (u8::MAX / 2) as f64) as u8,
            )
        }
    }
}

pub fn help() -> ! {
    println!("How to use:");
    println!("sstv [PATH TO IMAGE] [SAVE DESTINATION] [SSTV MODE]");
    println!();
    println!("[PATH TO IMAGE]");
    println!("\tSupply the relative or absolute path to the file in question to be converted.");
    println!();
    println!("[SAVE DESTINATION] (optional)");
    println!("\tSupply the relative or absolute path to the destination of the converted file.");
    println!();
    println!("[SSTV MODE] (optional)");
    println!(
        "\tIn what mode should this image be transmitted? If this is not specified, it will be chosen depending on the image resolution."
    );
    println!("\tMartinM1");
    println!("\tMartinM2");
    println!("\tMartinM3");
    println!("\tMartinM4");
    println!();
    println!("\tScottieS1");
    println!("\tScottieS2");
    println!("\tScottieS3");
    println!("\tScottieS4");
    println!("\tScottieDX");
    println!();
    println!("\tRobotBW8");
    println!();
    println!("\tWraaseSC2-180");
    println!();
    std::process::exit(1);
}
