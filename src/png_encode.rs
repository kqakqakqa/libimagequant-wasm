//! Indexed PNG writing shared by the static encoders in `lib.rs` and
//! `png_input.rs`. Only the `Encoder` construction differs between them.
use png::{BitDepth, ColorType, Encoder, Writer};
use std::io::Write;

/// Smallest bit depth that can address `colors` palette entries, paired with the
/// corresponding header value.
///
/// 1 bit covers 2 colors, 2 bits cover 4, 4 bits cover 16, and everything larger
/// needs 8. Padding a small palette to 8 bits multiplies the index plane for no
/// gain.
pub(crate) fn bit_depth_for(colors: usize) -> (u8, BitDepth) {
    match colors {
        0..=2 => (1, BitDepth::One),
        3..=4 => (2, BitDepth::Two),
        5..=16 => (4, BitDepth::Four),
        _ => (8, BitDepth::Eight),
    }
}

/// Applies the settings every indexed output uses.
pub(crate) fn configure<W: Write>(encoder: &mut Encoder<W>) {
    encoder.set_color(ColorType::Indexed);
    // The default trades file size for speed.
    encoder.set_compression(png::Compression::High);
    // No filter: palette indices carry no ordering, so differencing adds noise.
    encoder.set_filter(png::Filter::NoFilter);
}

/// Writes the index plane.
///
/// Sub-byte depths are packed, one byte-aligned run per scanline: the trailing
/// bits of a row are padding rather than the next row's first pixels.
pub(crate) fn write_indices<W: Write>(
    writer: &mut Writer<W>,
    bits: u8,
    indices: &[u8],
    width: usize,
    height: usize,
) -> Result<(), String> {
    if bits == 8 {
        return writer.write_image_data(indices).map_err(|e| e.to_string());
    }

    let row_bytes = (width * bits as usize + 7) / 8;
    let mut packed = vec![0u8; row_bytes * height];
    for y in 0..height {
        for x in 0..width {
            let index = indices[y * width + x] as usize;
            let bit_pos = x * bits as usize;
            packed[y * row_bytes + bit_pos / 8] |=
                (index as u8) << (8 - bits as usize - (bit_pos % 8));
        }
    }
    writer.write_image_data(&packed).map_err(|e| e.to_string())
}
