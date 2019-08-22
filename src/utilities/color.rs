#[allow(dead_code)]
pub struct Color {
    r: f32,
    b: f32,
    g: f32,
}

impl Color {
    #[allow(dead_code)]
    pub fn new(r: f32, b: f32, g: f32) -> Self {
        Color { r, g, b }
    }
    
    #[allow(dead_code)]
    pub fn from_u8(r: u8, g: u8, b: u8) -> Self {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
        }
    }

    #[allow(dead_code)]
    pub fn into_raw_f32(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }

    #[allow(dead_code)]
    pub fn into_raw_u32(self) -> [u32; 3] {
        [
            Color::into_linear(self.r).to_bits(),
            Color::into_linear(self.g).to_bits(),
            Color::into_linear(self.b).to_bits(),
        ]
    }

    pub fn into_linear(number: f32) -> f32 {
        number.powf(2.2)
    }
}
