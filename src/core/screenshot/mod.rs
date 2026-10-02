// (C) 2026 - Enzo Lombardi

//! Screenshot rendering: turn the terminal cell buffer into a PNG image.
//!
//! This module renders the in-memory screen buffer (a grid of [`Cell`]s, each
//! with a character and foreground/background colors) into a true-color PNG.
//! Text glyphs are drawn from an embedded 8x16 bitmap font; the box-drawing,
//! block and shade characters used by Turbo Vision frames are rendered
//! procedurally so they stay crisp at any cell size and tile across cells.
//! CP437's other symbols, the Latin-1 Supplement and the marks the framework
//! draws (`√`, `◆`, `✓`, ...) come from a table of hand-drawn bitmaps and
//! accented font letters. Any other character is drawn as `?`.
//!
//! The PNG encoder is fully self-contained (no external crates): it emits a
//! valid RGB PNG using uncompressed ("stored") DEFLATE blocks, so the produced
//! files are a little larger than a compressed encoder would make them but are
//! readable by every PNG viewer.
//!
//! # Example
//!
//! ```no_run
//! use turbo_vision::terminal::Terminal;
//!
//! let terminal = Terminal::init().unwrap();
//! // ... draw some UI ...
//! terminal.save_screenshot_png("screenshot.png").unwrap();
//! ```

#![allow(
    clippy::cast_possible_truncation,
    clippy::trivially_copy_pass_by_ref,
    reason = "Byte/pixel math in a self-contained PNG encoder narrows widths deliberately, and fixed-size chunk tags are passed by reference for call-site clarity."
)]

mod glyphs;
mod procedural;

use super::draw::Cell;
use super::palette::Attr;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

/// Embedded 8x16 bitmap font covering printable ASCII (`0x20..=0x7E`).
///
/// Layout: 16 bytes per glyph, one byte per scan-line, the most-significant
/// bit being the left-most pixel. This is the `Spleen` 8x16 bitmap font
/// (BSD-2-Clause) by Frederic Cambus — a font designed natively at 8x16, so the
/// glyphs are pixel-perfect with no rasterization. See `fonts/Spleen-LICENSE`.
static FONT_8X16: &[u8] = include_bytes!("font8x16.bin");

/// Glyph cell width of the embedded font, in pixels.
const FONT_W: usize = 8;
/// Glyph cell height of the embedded font, in pixels.
const FONT_H: usize = 16;
/// First codepoint present in [`FONT_8X16`].
const FONT_FIRST: u32 = 0x20;
/// Last codepoint present in [`FONT_8X16`].
const FONT_LAST: u32 = 0x7E;

/// A single glyph rendered as 16 rows of 8 horizontal pixels (bit 7 = left).
type GlyphMask = [u8; FONT_H];

/// Glyph height in pixels, exposed so callers can derive an integer scale.
pub const GLYPH_HEIGHT: usize = FONT_H;
/// Glyph width in pixels.
pub const GLYPH_WIDTH: usize = FONT_W;

/// Default cell used for buffer positions that are missing (out of range).
fn blank_cell() -> Cell {
    Cell::new(' ', Attr::from_u8(0x07))
}

/// Render the screen buffer to a PNG file at `path`.
///
/// * `buffer` - row-major grid of cells (`buffer[y][x]`).
/// * `cols` / `rows` - logical size of the screen in character cells.
/// * `scale` - integer magnification (clamped to at least 1). Each cell is
///   drawn as `GLYPH_WIDTH*scale` x `GLYPH_HEIGHT*scale` pixels. Using an
///   *integer* factor keeps glyphs crisp and preserves both the font's
///   proportions and the tiling of box-drawing characters (non-uniform scaling
///   would distort glyph shapes and inter-character spacing).
///
/// # Errors
///
/// Returns an error if the file cannot be created or written.
pub fn render_to_png(
    buffer: &[Vec<Cell>],
    cols: usize,
    rows: usize,
    scale: usize,
    path: &Path,
) -> io::Result<()> {
    let scale = scale.max(1);
    let cell_w = FONT_W * scale;
    let cell_h = FONT_H * scale;
    let img_w = cols * cell_w;
    let img_h = rows * cell_h;
    let mut rgb = vec![0u8; img_w * img_h * 3];

    for ry in 0..rows {
        let row = buffer.get(ry);
        for cx in 0..cols {
            let cell = row
                .and_then(|r| r.get(cx))
                .copied()
                .unwrap_or_else(blank_cell);
            let (fr, fg, fb) = cell.attr.fg.to_rgb();
            let (br, bg, bb) = cell.attr.bg.to_rgb();
            let mask = glyph_mask(cell.ch);

            for (gy, &row_bits) in mask.iter().enumerate() {
                for gx in 0..FONT_W {
                    let on = (row_bits >> (7 - gx)) & 1 != 0;
                    let (r, g, b) = if on { (fr, fg, fb) } else { (br, bg, bb) };
                    // Emit a scale x scale block of this source pixel.
                    let base_x = (cx * FONT_W + gx) * scale;
                    let base_y = (ry * FONT_H + gy) * scale;
                    for sy in 0..scale {
                        let oy = base_y + sy;
                        for sx in 0..scale {
                            let ox = base_x + sx;
                            let idx = (oy * img_w + ox) * 3;
                            rgb[idx] = r;
                            rgb[idx + 1] = g;
                            rgb[idx + 2] = b;
                        }
                    }
                }
            }
        }
    }

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    write_png(&mut writer, img_w, img_h, &rgb)?;
    writer.flush()
}

/// Build the 8x16 pixel mask for a character.
///
/// Printable ASCII comes from the embedded font. CP437's box-drawing, block,
/// shade and arrow glyphs are drawn procedurally ([`procedural`]); its other
/// symbols, the Latin-1 letters and signs and the marks the framework draws
/// are hand-drawn bitmaps or font letters with an accent ([`glyphs`]).
/// Anything else falls back to a blank cell (whitespace/control) or `?`
/// (other glyphs).
fn glyph_mask(ch: char) -> GlyphMask {
    let cp = ch as u32;

    if (FONT_FIRST..=FONT_LAST).contains(&cp) {
        return font_glyph(cp);
    }

    if let Some(mask) = procedural::glyph(ch).or_else(|| glyphs::glyph(ch)) {
        return mask;
    }

    if ch == '\0' || ch.is_whitespace() || ch.is_control() {
        [0u8; FONT_H]
    } else {
        // Unknown printable glyph: show a question mark so content stays visible.
        font_glyph('?' as u32)
    }
}

/// Copy a glyph out of the embedded font. `cp` must be in `FONT_FIRST..=FONT_LAST`.
fn font_glyph(cp: u32) -> GlyphMask {
    let off = (cp - FONT_FIRST) as usize * FONT_H;
    let mut mask = [0u8; FONT_H];
    mask.copy_from_slice(&FONT_8X16[off..off + FONT_H]);
    mask
}

// ----------------------------------------------------------------------------
// Minimal self-contained PNG encoder (RGB, 8-bit, stored DEFLATE)
// ----------------------------------------------------------------------------

/// CRC-32 lookup table (IEEE polynomial), computed at compile time.
const CRC_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut n = 0usize;
    while n < 256 {
        let mut c = n as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 {
                0xedb88320 ^ (c >> 1)
            } else {
                c >> 1
            };
            k += 1;
        }
        table[n] = c;
        n += 1;
    }
    table
};

/// Compute the PNG/zlib CRC-32 of a byte slice.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &b in bytes {
        crc = CRC_TABLE[((crc ^ u32::from(b)) & 0xff) as usize] ^ (crc >> 8);
    }
    crc ^ 0xffff_ffff
}

/// Compute the Adler-32 checksum used by the zlib wrapper.
fn adler32(bytes: &[u8]) -> u32 {
    const MOD: u32 = 65521;
    let mut a = 1u32;
    let mut b = 0u32;
    for &x in bytes {
        a = (a + u32::from(x)) % MOD;
        b = (b + a) % MOD;
    }
    (b << 16) | a
}

/// Write a single PNG chunk: length, type, data, CRC.
fn write_chunk<W: Write>(w: &mut W, kind: &[u8; 4], data: &[u8]) -> io::Result<()> {
    w.write_all(&(data.len() as u32).to_be_bytes())?;
    w.write_all(kind)?;
    w.write_all(data)?;
    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(kind);
    crc_input.extend_from_slice(data);
    w.write_all(&crc32(&crc_input).to_be_bytes())
}

/// Wrap raw bytes in a zlib stream using only uncompressed (stored) blocks.
fn zlib_stored(raw: &[u8]) -> Vec<u8> {
    // zlib header: CM=8/CINFO=7 (0x78), FLG chosen so (0x78<<8 | FLG) % 31 == 0.
    let mut out = vec![0x78u8, 0x01];
    let mut i = 0;
    if raw.is_empty() {
        // One empty final stored block.
        out.extend_from_slice(&[0x01, 0x00, 0x00, 0xff, 0xff]);
    }
    while i < raw.len() {
        let n = (raw.len() - i).min(0xffff);
        let is_final = i + n >= raw.len();
        out.push(u8::from(is_final)); // BFINAL bit, BTYPE = 00 (stored)
        let len = n as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(&raw[i..i + n]);
        i += n;
    }
    out.extend_from_slice(&adler32(raw).to_be_bytes());
    out
}

/// Encode `rgb` (width*height*3 bytes) as an 8-bit RGB PNG.
fn write_png<W: Write>(w: &mut W, width: usize, height: usize, rgb: &[u8]) -> io::Result<()> {
    // PNG signature.
    w.write_all(&[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a])?;

    // IHDR
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(2); // color type: truecolor RGB
    ihdr.push(0); // compression: deflate
    ihdr.push(0); // filter: adaptive
    ihdr.push(0); // interlace: none
    write_chunk(w, b"IHDR", &ihdr)?;

    // IDAT: prepend a filter byte (0 = none) to each scanline, then zlib-wrap.
    let stride = width * 3;
    let mut raw = Vec::with_capacity(height * (stride + 1));
    for y in 0..height {
        raw.push(0);
        raw.extend_from_slice(&rgb[y * stride..(y + 1) * stride]);
    }
    let idat = zlib_stored(&raw);
    write_chunk(w, b"IDAT", &idat)?;

    // IEND
    write_chunk(w, b"IEND", &[])
}

/// Every non-ASCII character the framework itself puts on screen, found by
/// scanning the non-test code in `src/` for `char` and string literals (and
/// the partial blocks `ProgressStyle::Smooth` computes from `U+2590 - n`).
/// `◆` is a common slider thumb. U+FE0F (after `ℹ` in the message-box
/// titles) is zero-width and never reaches a cell, so it is not listed.
#[cfg(test)]
const FRAMEWORK_GLYPHS: &str = "•ℹ→√─│┌┐└┘├┤┴═║╔╗╚╝▀▄█░■▲►▼◄◢⚠❌❓▏▎▍▌▋▊▉◆";

/// CP437's graphics characters as Unicode: the symbols in 0x01-0x1F and
/// 0x7F, and the letters and signs in 0x80-0xFF other than box drawing and
/// blocks (those are in [`CP437_BOX`] and [`CP437_BLOCKS`]).
#[cfg(test)]
const CP437_SYMBOLS: &str = concat!(
    "☺☻♥♦♣♠•◘○◙♂♀♪♫☼►◄↕‼¶§▬↨↑↓→←∟↔▲▼⌂",
    "ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáíóúñÑªº¿⌐¬½¼¡«»",
    "αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■",
);

/// CP437's box-drawing characters (single, double and mixed).
#[cfg(test)]
const CP437_BOX: &str = "─│┌┐└┘├┤┬┴┼═║╒╓╔╕╖╗╘╙╚╛╜╝╞╟╠╡╢╣╤╥╦╧╨╩╪╫╬";

/// CP437's block and shade characters.
#[cfg(test)]
const CP437_BLOCKS: &str = "▀▄█▌▐░▒▓";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::palette::TvColor;

    /// Panic with the characters that render as `?` or blank.
    fn assert_all_drawn(what: &str, chars: impl IntoIterator<Item = char>) {
        let question = glyph_mask('?');
        let missing: String = chars
            .into_iter()
            .filter(|&c| {
                let m = glyph_mask(c);
                m == question || m == [0u8; FONT_H]
            })
            .collect();
        assert!(
            missing.is_empty(),
            "{what} drawn as `?` or blank: {missing}"
        );
    }

    #[test]
    fn every_framework_glyph_has_a_bitmap() {
        assert_all_drawn("framework glyphs", FRAMEWORK_GLYPHS.chars());
        assert_all_drawn("marks", "◆✓▏▎▍▋▊▉".chars());
    }

    #[test]
    fn cp437_repertoire_has_bitmaps() {
        assert_all_drawn("CP437 symbols", CP437_SYMBOLS.chars());
        assert_all_drawn("CP437 box drawing", CP437_BOX.chars());
        assert_all_drawn("CP437 blocks", CP437_BLOCKS.chars());
    }

    #[test]
    fn whole_block_range_has_bitmaps() {
        assert_all_drawn(
            "blocks U+2580-U+259F",
            (0x2580..=0x259F).filter_map(char::from_u32),
        );
    }

    #[test]
    fn latin1_letters_have_bitmaps() {
        // U+00A0 (no-break space) is blank by design; everything after it
        // is a printable sign or letter.
        assert_all_drawn("Latin-1", (0xA1..=0xFF).filter_map(char::from_u32));
    }

    #[test]
    fn unknown_characters_still_fall_back_to_a_question_mark() {
        assert_eq!(glyph_mask('\u{4E2D}'), glyph_mask('?'));
        assert_eq!(glyph_mask('\u{1F600}'), glyph_mask('?'));
    }

    /// Weight of a box-drawing stroke at one cell edge.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum W {
        N,
        S,
        H,
        D,
    }

    /// The pixels a stroke of `w` sets along an edge 8 (columns) or 16
    /// (rows) pixels long; what a neighbour cell must match to join it.
    fn profile(w: W, len: usize) -> Vec<bool> {
        let on: &[usize] = match (w, len) {
            (W::N, _) => &[],
            (W::S, 8) => &[3, 4],
            (W::H, 8) => &[2, 3, 4, 5],
            (W::D, 8) => &[1, 2, 5, 6],
            (W::S, _) => &[7, 8],
            (W::H, _) => &[6, 7, 8, 9],
            (W::D, _) => &[5, 6, 9, 10],
        };
        (0..len).map(|i| on.contains(&i)).collect()
    }

    fn px(m: &GlyphMask, x: usize, y: usize) -> bool {
        m[y] >> (7 - x) & 1 != 0
    }

    #[test]
    fn box_drawing_joins_at_cell_edges() {
        use W::{D, H, N, S};
        // (char, up, down, left, right): every character of U+2500-U+257F
        // but the diagonals, whose strokes do not end on the rails.
        let cases = [
            ('─', N, N, S, S),
            ('━', N, N, H, H),
            ('│', S, S, N, N),
            ('┃', H, H, N, N),
            ('┄', N, N, S, S),
            ('┅', N, N, H, H),
            ('┆', S, S, N, N),
            ('┇', H, H, N, N),
            ('┈', N, N, S, S),
            ('┉', N, N, H, H),
            ('┊', S, S, N, N),
            ('┋', H, H, N, N),
            ('┌', N, S, N, S),
            ('┍', N, S, N, H),
            ('┎', N, H, N, S),
            ('┏', N, H, N, H),
            ('┐', N, S, S, N),
            ('┑', N, S, H, N),
            ('┒', N, H, S, N),
            ('┓', N, H, H, N),
            ('└', S, N, N, S),
            ('┕', S, N, N, H),
            ('┖', H, N, N, S),
            ('┗', H, N, N, H),
            ('┘', S, N, S, N),
            ('┙', S, N, H, N),
            ('┚', H, N, S, N),
            ('┛', H, N, H, N),
            ('├', S, S, N, S),
            ('┝', S, S, N, H),
            ('┞', H, S, N, S),
            ('┟', S, H, N, S),
            ('┠', H, H, N, S),
            ('┡', H, S, N, H),
            ('┢', S, H, N, H),
            ('┣', H, H, N, H),
            ('┤', S, S, S, N),
            ('┥', S, S, H, N),
            ('┦', H, S, S, N),
            ('┧', S, H, S, N),
            ('┨', H, H, S, N),
            ('┩', H, S, H, N),
            ('┪', S, H, H, N),
            ('┫', H, H, H, N),
            ('┬', N, S, S, S),
            ('┭', N, S, H, S),
            ('┮', N, S, S, H),
            ('┯', N, S, H, H),
            ('┰', N, H, S, S),
            ('┱', N, H, H, S),
            ('┲', N, H, S, H),
            ('┳', N, H, H, H),
            ('┴', S, N, S, S),
            ('┵', S, N, H, S),
            ('┶', S, N, S, H),
            ('┷', S, N, H, H),
            ('┸', H, N, S, S),
            ('┹', H, N, H, S),
            ('┺', H, N, S, H),
            ('┻', H, N, H, H),
            ('┼', S, S, S, S),
            ('┽', S, S, H, S),
            ('┾', S, S, S, H),
            ('┿', S, S, H, H),
            ('╀', H, S, S, S),
            ('╁', S, H, S, S),
            ('╂', H, H, S, S),
            ('╃', H, S, H, S),
            ('╄', H, S, S, H),
            ('╅', S, H, H, S),
            ('╆', S, H, S, H),
            ('╇', H, S, H, H),
            ('╈', S, H, H, H),
            ('╉', H, H, H, S),
            ('╊', H, H, S, H),
            ('╋', H, H, H, H),
            ('╌', N, N, S, S),
            ('╍', N, N, H, H),
            ('╎', S, S, N, N),
            ('╏', H, H, N, N),
            ('═', N, N, D, D),
            ('║', D, D, N, N),
            ('╒', N, S, N, D),
            ('╓', N, D, N, S),
            ('╔', N, D, N, D),
            ('╕', N, S, D, N),
            ('╖', N, D, S, N),
            ('╗', N, D, D, N),
            ('╘', S, N, N, D),
            ('╙', D, N, N, S),
            ('╚', D, N, N, D),
            ('╛', S, N, D, N),
            ('╜', D, N, S, N),
            ('╝', D, N, D, N),
            ('╞', S, S, N, D),
            ('╟', D, D, N, S),
            ('╠', D, D, N, D),
            ('╡', S, S, D, N),
            ('╢', D, D, S, N),
            ('╣', D, D, D, N),
            ('╤', N, S, D, D),
            ('╥', N, D, S, S),
            ('╦', N, D, D, D),
            ('╧', S, N, D, D),
            ('╨', D, N, S, S),
            ('╩', D, N, D, D),
            ('╪', S, S, D, D),
            ('╫', D, D, S, S),
            ('╬', D, D, D, D),
            ('╭', N, S, N, S),
            ('╮', N, S, S, N),
            ('╯', S, N, S, N),
            ('╰', S, N, N, S),
            ('╴', N, N, S, N),
            ('╵', S, N, N, N),
            ('╶', N, N, N, S),
            ('╷', N, S, N, N),
            ('╸', N, N, H, N),
            ('╹', H, N, N, N),
            ('╺', N, N, N, H),
            ('╻', N, H, N, N),
            ('╼', N, N, S, H),
            ('╽', S, H, N, N),
            ('╾', N, N, H, S),
            ('╿', H, S, N, N),
        ];
        let covered: String = cases.iter().map(|c| c.0).chain(DIAGONALS.chars()).collect();
        let block: String = (0x2500..=0x257F).filter_map(char::from_u32).collect();
        let mut sorted: Vec<char> = covered.chars().collect();
        sorted.sort_unstable();
        assert_eq!(sorted.into_iter().collect::<String>(), block);

        for (ch, up, down, left, right) in cases {
            let m = glyph_mask(ch);
            let top: Vec<bool> = (0..FONT_W).map(|x| px(&m, x, 0)).collect();
            let bottom: Vec<bool> = (0..FONT_W).map(|x| px(&m, x, FONT_H - 1)).collect();
            let west: Vec<bool> = (0..FONT_H).map(|y| px(&m, 0, y)).collect();
            let east: Vec<bool> = (0..FONT_H).map(|y| px(&m, FONT_W - 1, y)).collect();
            // A dashed line may have a gap on the cell edge.
            let joins = |edge: &[bool], w: W| {
                edge == profile(w, edge.len()).as_slice()
                    || (DASHED.contains(ch) && edge.iter().all(|&on| !on))
            };
            assert!(joins(&top, up), "{ch}: top edge {top:?}");
            assert!(joins(&bottom, down), "{ch}: bottom edge {bottom:?}");
            assert!(joins(&west, left), "{ch}: left edge {west:?}");
            assert!(joins(&east, right), "{ch}: right edge {east:?}");
            if DASHED.contains(ch) {
                let (a, b) = if up == N {
                    (&west, &east)
                } else {
                    (&top, &bottom)
                };
                assert!(
                    a.iter().chain(b).any(|&on| on),
                    "{ch}: no stroke on its edges"
                );
            }
        }
    }

    const DIAGONALS: &str = "╱╲╳";
    const DASHED: &str = "┄┅┆┇┈┉┊┋╌╍╎╏";

    /// The lengths of the runs of equal pixels along a cyclic line, from
    /// its first change: what a row of copies of the cell shows.
    fn cyclic_runs(line: &[bool]) -> Vec<(bool, usize)> {
        let start = (1..line.len())
            .find(|&i| line[i] != line[i - 1])
            .unwrap_or(0);
        let mut runs: Vec<(bool, usize)> = Vec::new();
        for i in 0..line.len() {
            let on = line[(start + i) % line.len()];
            match runs.last_mut() {
                Some((v, n)) if *v == on => *n += 1,
                _ => runs.push((on, 1)),
            }
        }
        runs
    }

    #[test]
    fn dashed_lines_tile_with_even_gaps() {
        // (dashed, solid, dashes per cell)
        for (dashed, solid, n) in [
            ('╌', '─', 2),
            ('╍', '━', 2),
            ('╎', '│', 2),
            ('╏', '┃', 2),
            ('┄', '─', 3),
            ('┅', '━', 3),
            ('┆', '│', 3),
            ('┇', '┃', 3),
            ('┈', '─', 4),
            ('┉', '━', 4),
            ('┊', '│', 4),
            ('┋', '┃', 4),
        ] {
            let (d, s) = (glyph_mask(dashed), glyph_mask(solid));
            // Segments of the solid stroke, nothing else.
            for y in 0..FONT_H {
                assert_eq!(d[y] & !s[y], 0, "{dashed}: row {y} leaves {solid}");
            }
            let horizontal = s[0] == 0;
            let (len, rail) = if horizontal {
                (FONT_W, s.iter().position(|&r| r != 0).unwrap())
            } else {
                (FONT_H, (0..FONT_W).find(|&x| px(&s, x, 0)).unwrap())
            };
            let line: Vec<bool> = (0..len)
                .map(|i| {
                    if horizontal {
                        px(&d, i, rail)
                    } else {
                        px(&d, rail, i)
                    }
                })
                .collect();
            // Every pixel across the stroke follows the same pattern.
            for y in 0..FONT_H {
                for x in 0..FONT_W {
                    let i = if horizontal { x } else { y };
                    assert_eq!(px(&d, x, y), px(&s, x, y) && line[i], "{dashed} ({x},{y})");
                }
            }
            let runs = cyclic_runs(&line);
            let gaps: Vec<usize> = runs.iter().filter(|r| !r.0).map(|r| r.1).collect();
            let dashes: Vec<usize> = runs.iter().filter(|r| r.0).map(|r| r.1).collect();
            assert_eq!(gaps.len(), n, "{dashed}: {runs:?}");
            assert!(
                gaps.iter().all(|&g| g == gaps[0]),
                "{dashed}: gaps {gaps:?}"
            );
            let (lo, hi) = (dashes.iter().min().unwrap(), dashes.iter().max().unwrap());
            assert!(hi - lo <= 1, "{dashed}: dashes {dashes:?}");
        }
    }

    #[test]
    fn rounded_corners_trim_the_light_corner() {
        for (round, square) in [('╭', '┌'), ('╮', '┐'), ('╯', '┘'), ('╰', '└')] {
            let (r, s) = (glyph_mask(round), glyph_mask(square));
            let removed: u32 = (0..FONT_H).map(|y| (s[y] & !r[y]).count_ones()).sum();
            let added: u32 = (0..FONT_H).map(|y| (r[y] & !s[y]).count_ones()).sum();
            assert_eq!((removed, added), (1, 0), "{round} against {square}");
        }
    }

    #[test]
    fn diagonals_run_corner_to_corner() {
        let back = glyph_mask('╲');
        let fwd = glyph_mask('╱');
        assert!(px(&back, 0, 0) && px(&back, FONT_W - 1, FONT_H - 1));
        assert!(!px(&back, FONT_W - 1, 0) && !px(&back, 0, FONT_H - 1));
        assert!(px(&fwd, FONT_W - 1, 0) && px(&fwd, 0, FONT_H - 1));
        assert!(!px(&fwd, 0, 0) && !px(&fwd, FONT_W - 1, FONT_H - 1));
        for y in 0..FONT_H {
            assert_eq!(fwd[y], back[y].reverse_bits(), "╱ mirrors ╲ on row {y}");
            assert_eq!(glyph_mask('╳')[y], fwd[y] | back[y], "╳ row {y}");
        }
        // One unbroken line: each row touches the row above it.
        for y in 1..FONT_H {
            let spread = back[y - 1] | back[y - 1] << 1 | back[y - 1] >> 1;
            assert!(back[y] != 0 && back[y] & spread != 0, "╲ breaks at row {y}");
        }
    }

    #[test]
    fn whole_box_drawing_range_has_bitmaps() {
        assert_all_drawn(
            "box drawing U+2500-U+257F",
            (0x2500..=0x257F).filter_map(char::from_u32),
        );
    }

    #[test]
    fn double_tees_open_the_rail_they_branch_from() {
        // ╠'s right rail turns into the branch: no pixel between the two
        // horizontal rails on it, unlike ╟, whose rail runs straight down.
        let tee = glyph_mask('╠');
        assert!(!px(&tee, 5, 7) && !px(&tee, 5, 8), "╠ inner rail is open");
        let single = glyph_mask('╟');
        assert!(px(&single, 5, 7) && px(&single, 5, 8), "╟ rail is closed");
    }

    #[test]
    fn distinct_glyphs_look_distinct() {
        for (a, b) in [
            ('►', '◄'),
            ('▲', '▼'),
            ('é', 'e'),
            ('è', 'é'),
            ('ì', 'i'),
            ('Å', 'A'),
            ('▏', '▎'),
            ('▍', '▌'),
            ('½', '¼'),
            ('≤', '≥'),
            ('♂', '♀'),
            ('√', '✓'),
            ('◆', '♦'),
            ('❓', '?'),
            ('━', '─'),
            ('┃', '│'),
            ('┏', '┌'),
            ('╋', '┼'),
            ('┍', '┎'),
            ('┄', '─'),
            ('┅', '━'),
            ('┆', '│'),
            ('╌', '┄'),
            ('┈', '┄'),
            ('╭', '┌'),
            ('╱', '╲'),
        ] {
            assert_ne!(glyph_mask(a), glyph_mask(b), "{a} and {b} look the same");
        }
    }

    #[test]
    fn eighth_blocks_grow_by_one_column() {
        // ▏ ▎ ▍ ▌ ▋ ▊ ▉ █: 1..=8 columns from the left.
        for (n, ch) in "▏▎▍▌▋▊▉█".chars().enumerate() {
            let row = (0xFF00u16 >> (n + 1)) as u8;
            assert_eq!(glyph_mask(ch), [row; FONT_H], "{ch}");
        }
    }

    #[test]
    fn renders_the_new_glyphs_at_the_usual_size() {
        let attr = Attr::new(TvColor::White, TvColor::Blue);
        let row: Vec<Cell> = FRAMEWORK_GLYPHS
            .chars()
            .chain(CP437_SYMBOLS.chars())
            .chain(CP437_BOX.chars())
            .map(|c| Cell::new(c, attr))
            .collect();
        let cols = row.len();
        let path = std::env::temp_dir().join("tv_screenshot_glyphs_test.png");
        render_to_png(&[row], cols, 1, 2, &path).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(&bytes[16..20], &((cols * FONT_W * 2) as u32).to_be_bytes());
        assert_eq!(&bytes[20..24], &((FONT_H * 2) as u32).to_be_bytes());
    }

    #[test]
    fn crc32_known_value() {
        // CRC-32 of "IEND" is a well-known constant.
        assert_eq!(crc32(b"IEND"), 0xae42_6082);
    }

    #[test]
    fn adler32_known_value() {
        // Adler-32 of a single zero byte: a=1, b=1.
        assert_eq!(adler32(&[0]), 0x0001_0001);
    }

    #[test]
    fn ascii_glyph_is_nonblank() {
        // 'A' must have some set pixels.
        let m = glyph_mask('A');
        assert!(m.iter().any(|&row| row != 0));
    }

    #[test]
    fn space_is_blank() {
        assert_eq!(glyph_mask(' '), [0u8; FONT_H]);
    }

    #[test]
    fn full_block_is_solid() {
        assert_eq!(glyph_mask('█'), [0xffu8; FONT_H]);
    }

    #[test]
    fn writes_valid_png_header() {
        let buffer = vec![vec![
            Cell::new('H', Attr::new(TvColor::White, TvColor::Blue)),
            Cell::new('i', Attr::new(TvColor::White, TvColor::Blue)),
        ]];
        let dir = std::env::temp_dir();
        let path = dir.join("tv_screenshot_test.png");
        render_to_png(&buffer, 2, 1, 1, &path).unwrap();

        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(
            &bytes[..8],
            &[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]
        );
        // IHDR width/height live at byte offset 16..24.
        assert_eq!(&bytes[16..20], &(16u32).to_be_bytes()); // 2 cols * 8px * 1
        assert_eq!(&bytes[20..24], &(16u32).to_be_bytes()); // 1 row * 16px * 1
        let _ = std::fs::remove_file(&path);
    }
}
