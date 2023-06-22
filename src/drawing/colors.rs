#[repr(C)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RGB<ComponentType> {
    /// Red
    pub r: ComponentType,
    /// Green
    pub g: ComponentType,
    /// Blue
    pub b: ComponentType,
}

impl<T> RGB<T> {
    #[inline(always)]
    pub const fn new(r: T, g: T, b: T) -> Self {
        Self { r, g, b }
    }
}

pub fn random_color() -> u32 {
    winapi::um::wingdi::RGB(
        rand::random::<u8>(),
        rand::random::<u8>(),
        rand::random::<u8>(),
    )
}

pub fn interpolate_colors(colors: &[RGB<u8>], weight: f32) -> u32 {
    let num_colors = colors.len();
    let segment = 1.0 / (num_colors - 1) as f32;

    // Find the two adjacent colors for the given weight
    let index1 = (weight / segment).floor() as usize;
    let index2 = index1 + 1;

    let color1 = colors[index1];
    let color2 = colors[index2];

    // Calculate the weight within the segment
    let segment_weight = (weight - index1 as f32 * segment) / segment;

    // Interpolate between the two colors
    let r = ((1.0 - segment_weight) * color1.r as f32 + segment_weight * color2.r as f32) as u8;
    let g = ((1.0 - segment_weight) * color1.g as f32 + segment_weight * color2.g as f32) as u8;
    let b = ((1.0 - segment_weight) * color1.b as f32 + segment_weight * color2.b as f32) as u8;

    winapi::um::wingdi::RGB(r, g, b)
}

pub fn interpolate_floats(floats: &[f32], weight: f32) -> f32 {
    let num_colors = floats.len();
    let segment = 1.0 / (num_colors - 1) as f32;

    // Find the two adjacent colors for the given weight
    let index1 = (weight / segment).floor() as usize;
    let index2 = index1 + 1;

    let float1 = floats[index1];
    let float2 = floats[index2];

    // Calculate the weight within the segment
    let segment_weight = (weight - index1 as f32 * segment) / segment;

    // Interpolate between the two floats
    (1.0 - segment_weight) * float1 as f32 + segment_weight * float2 as f32
}