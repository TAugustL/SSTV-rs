#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum SSTVMode {
    // Martin
    MartinM1,
    MartinM2,
    MartinM3,
    MartinM4,

    // Scottie
    ScottieS1,
    ScottieS2,
    ScottieS3,
    ScottieS4,
    ScottieDX,

    // B/W Mode
    BWSC1s8,

    // Wraase
    WraaseSC2s180,
}

impl SSTVMode {
    pub fn is_scottie(&self) -> bool {
        [
            SSTVMode::ScottieS1,
            SSTVMode::ScottieS2,
            SSTVMode::ScottieS3,
            SSTVMode::ScottieS4,
            SSTVMode::ScottieDX,
        ]
        .contains(self)
    }
    pub fn is_martin(&self) -> bool {
        [
            SSTVMode::MartinM1,
            SSTVMode::MartinM2,
            SSTVMode::MartinM3,
            SSTVMode::MartinM4,
        ]
        .contains(self)
    }
    pub fn is_bw(&self) -> bool {
        [SSTVMode::BWSC1s8].contains(self)
    }
    pub fn is_wraase(&self) -> bool {
        [SSTVMode::WraaseSC2s180].contains(self)
    }
}

pub fn get_mode_from_resolution(w: u32, h: u32, monochrome: bool) -> SSTVMode {
    const RATIOS: [f32; 7] = [0.625, 1.0, 1.25, 1.3333, 2.0, 2.5, 2.6667];
    const AREAS: [u32; 10] = [
        16384, 19200, 20480, 32768, 38400, 40960, 65536, 76800, 81920, 131072,
    ];

    let ratio = w as f32 / h as f32;
    let area = w * h;
    let mode: SSTVMode;

    let mut closest_r: f32 = 1000.0;
    for r in RATIOS {
        if (ratio - r).abs() < closest_r {
            closest_r = r;
        }
    }

    let mut closest_a: u32 = 1000000;
    for a in AREAS {
        if (area).abs_diff(a) < closest_a {
            closest_a = a;
        }
    }

    if monochrome {
        mode = SSTVMode::BWSC1s8;
    } else {
        match closest_r {
            0.625 => mode = SSTVMode::MartinM2,
            1.25 => match closest_a {
                20480 => mode = SSTVMode::ScottieS4,
                81920 => mode = SSTVMode::ScottieS1,
                _ => mode = SSTVMode::MartinM4,
            },
            2.0 => mode = SSTVMode::WraaseSC2s180,
            2.5 => match closest_a {
                40960 => mode = SSTVMode::ScottieS2,
                _ => mode = SSTVMode::MartinM2,
            },
            _ => {
                mode = SSTVMode::ScottieS1;
            }
        }
    }

    mode
}
