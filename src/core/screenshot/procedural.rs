// (C) 2026 - Enzo Lombardi

//! Glyphs drawn from geometry rather than bitmaps: box drawing
//! (U+2500-U+257F), the block elements (U+2580-U+259F), shades, and the
//! triangles Turbo Vision uses for arrows and the resize handle.
//!
//! Box-drawing strokes run from the cell edge to the middle of the cell on
//! fixed rails, so a stroke leaving one cell meets the stroke entering the
//! next whatever the two characters are. Dashed lines are the solid stroke
//! with gaps, rounded corners are light corners with the corner pixel cut,
//! and the diagonals run corner to corner.

use super::{FONT_H, FONT_W, GlyphMask};

/// Set pixel (x, y) in a mask (no-op if out of the 8x16 bounds).
fn set_px(mask: &mut GlyphMask, x: usize, y: usize) {
    if x < FONT_W && y < FONT_H {
        mask[y] |= 1 << (7 - x);
    }
}

/// Fill a rectangular pixel region `[x0, x1) x [y0, y1)`.
fn fill_rect(mask: &mut GlyphMask, x0: usize, y0: usize, x1: usize, y1: usize) {
    for y in y0..y1 {
        for x in x0..x1 {
            set_px(mask, x, y);
        }
    }
}

/// Return a procedurally-rendered glyph, or `None` if `ch` is not one of the
/// characters drawn here.
pub(super) fn glyph(ch: char) -> Option<GlyphMask> {
    if let Some(arms) = box_arms(ch) {
        return Some(box_glyph(arms));
    }
    if let Some((solid, dashes)) = dashed(ch) {
        return Some(dash(box_arms(solid)?, dashes));
    }
    if let Some(square) = rounded(ch) {
        return Some(round_corner(box_arms(square)?));
    }
    if let Some(mask) = diagonal(ch) {
        return Some(mask);
    }
    if let Some(mask) = block(ch) {
        return Some(mask);
    }
    let mask = match ch {
        // Small filled square
        '■' => {
            let mut m = [0u8; FONT_H];
            fill_rect(&mut m, 1, 4, FONT_W - 1, FONT_H - 4);
            m
        }
        // Arrows
        '▲' => triangle(Dir::Up),
        '▼' => triangle(Dir::Down),
        '◄' => triangle(Dir::Left),
        '►' => triangle(Dir::Right),
        // Corner triangles (e.g. window resize handle ◢)
        '◢' => corner_triangle(Corner::LowerRight),
        '◣' => corner_triangle(Corner::LowerLeft),
        '◤' => corner_triangle(Corner::UpperLeft),
        '◥' => corner_triangle(Corner::UpperRight),
        _ => return None,
    };
    Some(mask)
}

// ----------------------------------------------------------------------------
// Box drawing
// ----------------------------------------------------------------------------

/// The stroke a box-drawing character has in one direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum W {
    /// No stroke.
    N,
    /// A single (light) line.
    S,
    /// A heavy line: one wide rail centred on the single line's.
    H,
    /// A double line.
    D,
}

/// The strokes of a box-drawing character: up, down, left, right.
#[derive(Clone, Copy, Debug)]
struct Arms {
    up: W,
    down: W,
    left: W,
    right: W,
}

/// The solid box-drawing characters, by the strokes they have: light,
/// heavy and double lines, their corners, tees and crosses, and the half
/// lines.
fn box_arms(ch: char) -> Option<Arms> {
    use W::{D, H, N, S};
    let (up, down, left, right) = match ch {
        '─' => (N, N, S, S),
        '━' => (N, N, H, H),
        '│' => (S, S, N, N),
        '┃' => (H, H, N, N),
        '┌' => (N, S, N, S),
        '┍' => (N, S, N, H),
        '┎' => (N, H, N, S),
        '┏' => (N, H, N, H),
        '┐' => (N, S, S, N),
        '┑' => (N, S, H, N),
        '┒' => (N, H, S, N),
        '┓' => (N, H, H, N),
        '└' => (S, N, N, S),
        '┕' => (S, N, N, H),
        '┖' => (H, N, N, S),
        '┗' => (H, N, N, H),
        '┘' => (S, N, S, N),
        '┙' => (S, N, H, N),
        '┚' => (H, N, S, N),
        '┛' => (H, N, H, N),
        '├' => (S, S, N, S),
        '┝' => (S, S, N, H),
        '┞' => (H, S, N, S),
        '┟' => (S, H, N, S),
        '┠' => (H, H, N, S),
        '┡' => (H, S, N, H),
        '┢' => (S, H, N, H),
        '┣' => (H, H, N, H),
        '┤' => (S, S, S, N),
        '┥' => (S, S, H, N),
        '┦' => (H, S, S, N),
        '┧' => (S, H, S, N),
        '┨' => (H, H, S, N),
        '┩' => (H, S, H, N),
        '┪' => (S, H, H, N),
        '┫' => (H, H, H, N),
        '┬' => (N, S, S, S),
        '┭' => (N, S, H, S),
        '┮' => (N, S, S, H),
        '┯' => (N, S, H, H),
        '┰' => (N, H, S, S),
        '┱' => (N, H, H, S),
        '┲' => (N, H, S, H),
        '┳' => (N, H, H, H),
        '┴' => (S, N, S, S),
        '┵' => (S, N, H, S),
        '┶' => (S, N, S, H),
        '┷' => (S, N, H, H),
        '┸' => (H, N, S, S),
        '┹' => (H, N, H, S),
        '┺' => (H, N, S, H),
        '┻' => (H, N, H, H),
        '┼' => (S, S, S, S),
        '┽' => (S, S, H, S),
        '┾' => (S, S, S, H),
        '┿' => (S, S, H, H),
        '╀' => (H, S, S, S),
        '╁' => (S, H, S, S),
        '╂' => (H, H, S, S),
        '╃' => (H, S, H, S),
        '╄' => (H, S, S, H),
        '╅' => (S, H, H, S),
        '╆' => (S, H, S, H),
        '╇' => (H, S, H, H),
        '╈' => (S, H, H, H),
        '╉' => (H, H, H, S),
        '╊' => (H, H, S, H),
        '╋' => (H, H, H, H),
        '═' => (N, N, D, D),
        '║' => (D, D, N, N),
        '╒' => (N, S, N, D),
        '╓' => (N, D, N, S),
        '╔' => (N, D, N, D),
        '╕' => (N, S, D, N),
        '╖' => (N, D, S, N),
        '╗' => (N, D, D, N),
        '╘' => (S, N, N, D),
        '╙' => (D, N, N, S),
        '╚' => (D, N, N, D),
        '╛' => (S, N, D, N),
        '╜' => (D, N, S, N),
        '╝' => (D, N, D, N),
        '╞' => (S, S, N, D),
        '╟' => (D, D, N, S),
        '╠' => (D, D, N, D),
        '╡' => (S, S, D, N),
        '╢' => (D, D, S, N),
        '╣' => (D, D, D, N),
        '╤' => (N, S, D, D),
        '╥' => (N, D, S, S),
        '╦' => (N, D, D, D),
        '╧' => (S, N, D, D),
        '╨' => (D, N, S, S),
        '╩' => (D, N, D, D),
        '╪' => (S, S, D, D),
        '╫' => (D, D, S, S),
        '╬' => (D, D, D, D),
        '╴' => (N, N, S, N),
        '╵' => (S, N, N, N),
        '╶' => (N, N, N, S),
        '╷' => (N, S, N, N),
        '╸' => (N, N, H, N),
        '╹' => (H, N, N, N),
        '╺' => (N, N, N, H),
        '╻' => (N, H, N, N),
        '╼' => (N, N, S, H),
        '╽' => (S, H, N, N),
        '╾' => (N, N, H, S),
        '╿' => (H, S, N, N),
        _ => return None,
    };
    Some(Arms {
        up,
        down,
        left,
        right,
    })
}

/// The dashed lines: the solid line each is cut from, and how many dashes
/// it has per cell.
fn dashed(ch: char) -> Option<(char, usize)> {
    Some(match ch {
        '╌' => ('─', 2),
        '╍' => ('━', 2),
        '╎' => ('│', 2),
        '╏' => ('┃', 2),
        '┄' => ('─', 3),
        '┅' => ('━', 3),
        '┆' => ('│', 3),
        '┇' => ('┃', 3),
        '┈' => ('─', 4),
        '┉' => ('━', 4),
        '┊' => ('│', 4),
        '┋' => ('┃', 4),
        _ => return None,
    })
}

/// Cut `dashes` evenly spaced gaps into a straight line. The gaps are an
/// eighth of the cell's length along the line (one pixel across, two
/// down), centred in each `1/dashes` of it, so the pattern repeats from
/// one cell to the next.
fn dash(arms: Arms, dashes: usize) -> GlyphMask {
    let mut mask = box_glyph(arms);
    let horizontal = arms.left != W::N;
    let len = if horizontal { FONT_W } else { FONT_H };
    let gap = len / 8;
    for i in 0..dashes {
        // The centre of the i-th stretch, less half a gap, rounded.
        let start = ((2 * i + 1) * len + dashes - gap * dashes) / (2 * dashes);
        for p in start..start + gap {
            if horizontal {
                for row in &mut mask {
                    *row &= !(1 << (7 - p));
                }
            } else {
                mask[p] = 0;
            }
        }
    }
    mask
}

/// The rounded corners, by the square light corner each rounds.
fn rounded(ch: char) -> Option<char> {
    Some(match ch {
        '╭' => '┌',
        '╮' => '┐',
        '╯' => '┘',
        '╰' => '└',
        _ => return None,
    })
}

/// A light corner with the pixel at the outside of the bend cut away.
fn round_corner(arms: Arms) -> GlyphMask {
    let mut mask = box_glyph(arms);
    let (x0, x1) = columns(W::S)[0];
    let (y0, y1) = rows(W::S)[0];
    let x = if arms.right != W::N { x0 } else { x1 - 1 };
    let y = if arms.down != W::N { y0 } else { y1 - 1 };
    mask[y] &= !(1 << (7 - x));
    mask
}

/// The diagonals, corner to corner. A line two pixels wide across, about
/// as heavy as a light stroke, runs from the top-left pixel to the
/// bottom-right one; `╱` is its mirror image and `╳` both.
fn diagonal(ch: char) -> Option<GlyphMask> {
    let back = shade(|x, y| {
        // Pixel centres within a pixel of the line x = (y + 1/2) / 2.
        let d = 4 * x as isize - 2 * y as isize + 1;
        d.abs() < 4
    });
    let fwd = back.map(u8::reverse_bits);
    Some(match ch {
        '╲' => back,
        '╱' => fwd,
        '╳' => std::array::from_fn(|y| back[y] | fwd[y]),
        _ => return None,
    })
}

/// The columns `[x0, x1)` of a vertical stroke: a single line on columns
/// 3-4, a heavy one on 2-5, a double line on 1-2 and 5-6.
fn columns(w: W) -> &'static [(usize, usize)] {
    match w {
        W::N => &[],
        W::S => &[(3, 5)],
        W::H => &[(2, 6)],
        W::D => &[(1, 3), (5, 7)],
    }
}

/// The rows `[y0, y1)` of a horizontal stroke: a single line on rows 7-8,
/// a heavy one on 6-9, a double line on 5-6 and 9-10.
fn rows(w: W) -> &'static [(usize, usize)] {
    match w {
        W::N => &[],
        W::S => &[(7, 9)],
        W::H => &[(6, 10)],
        W::D => &[(5, 7), (9, 11)],
    }
}

/// One arm of a box-drawing character, described along its own axis: how
/// far each of its rails runs from the cell edge it starts at.
struct Arm {
    /// This arm's stroke.
    w: W,
    /// The stroke of the arm opposite it.
    opposite: W,
    /// The strokes of the two arms across it: the one on the side of its
    /// first rail, then the one on the side of its second.
    sides: [W; 2],
    /// The rails, as cross-axis ranges, of a stroke of a given weight
    /// along this arm's axis or across it.
    own: fn(W) -> &'static [(usize, usize)],
    across: fn(W) -> &'static [(usize, usize)],
    /// The cell's length along this arm's axis.
    len: usize,
}

impl Arm {
    /// Where each rail of an arm that starts at the near edge (up, left)
    /// ends, paired with the rail's cross-axis range.
    fn near_edge_rails(&self) -> Vec<((usize, usize), usize)> {
        let center = (self.across)(W::S)[0];
        let crossing: Vec<&[(usize, usize)]> = self
            .sides
            .iter()
            .filter(|&&s| s != W::N)
            .map(|&s| (self.across)(s))
            .collect();
        let first_end = |l: &[(usize, usize)]| l[0].1;
        let last_end = |l: &[(usize, usize)]| l[l.len() - 1].1;
        let rails = (self.own)(self.w);
        if self.w != W::D {
            // One rail (light or heavy): it ends where it meets the strokes
            // across it.
            let end = if self.opposite != W::N {
                center.1
            } else if crossing.len() == 2 {
                // A tee: meet the nearer line of the stroke across.
                crossing
                    .iter()
                    .map(|l| first_end(l))
                    .max()
                    .unwrap_or(center.1)
            } else if crossing.len() == 1 {
                // A corner: reach the far line of the stroke it turns into.
                last_end(crossing[0])
            } else {
                center.1
            };
            return vec![(rails[0], end)];
        }
        rails
            .iter()
            .zip(self.sides)
            .map(|(&rail, side)| {
                let end = if side != W::N {
                    // The rail turns into the stroke on its side.
                    first_end((self.across)(side))
                } else if self.opposite == W::D || (crossing.is_empty() && self.opposite != W::N) {
                    self.len
                } else if crossing.is_empty() {
                    center.1
                } else {
                    // The outer rail of a corner or tee runs to the far line.
                    crossing
                        .iter()
                        .map(|l| last_end(l))
                        .max()
                        .unwrap_or(center.1)
                };
                (rail, end)
            })
            .collect()
    }

    /// Where each rail of an arm that ends at the far edge (down, right)
    /// starts, paired with the rail's cross-axis range.
    fn far_edge_rails(&self) -> Vec<((usize, usize), usize)> {
        let center = (self.across)(W::S)[0];
        let crossing: Vec<&[(usize, usize)]> = self
            .sides
            .iter()
            .filter(|&&s| s != W::N)
            .map(|&s| (self.across)(s))
            .collect();
        let first_start = |l: &[(usize, usize)]| l[0].0;
        let last_start = |l: &[(usize, usize)]| l[l.len() - 1].0;
        let rails = (self.own)(self.w);
        if self.w != W::D {
            // One rail (light or heavy): it ends where it meets the strokes
            // across it.
            let start = if self.opposite != W::N {
                center.0
            } else if crossing.len() == 2 {
                crossing
                    .iter()
                    .map(|l| last_start(l))
                    .min()
                    .unwrap_or(center.0)
            } else if crossing.len() == 1 {
                first_start(crossing[0])
            } else {
                center.0
            };
            return vec![(rails[0], start)];
        }
        rails
            .iter()
            .zip(self.sides)
            .map(|(&rail, side)| {
                let start = if side != W::N {
                    last_start((self.across)(side))
                } else if self.opposite == W::D || (crossing.is_empty() && self.opposite != W::N) {
                    0
                } else if crossing.is_empty() {
                    center.0
                } else {
                    crossing
                        .iter()
                        .map(|l| first_start(l))
                        .min()
                        .unwrap_or(center.0)
                };
                (rail, start)
            })
            .collect()
    }
}

/// Draw a box-drawing character from its four strokes.
///
/// Each stroke meets the cell edge on its rails. Where strokes meet, a
/// single or heavy line stops at the nearer line it joins, and the rails of a double
/// line turn into the stroke on their side or, with none there, run on to
/// the far line, so corners and tees join the way CP437 draws them.
fn box_glyph(a: Arms) -> GlyphMask {
    let mut mask = [0u8; FONT_H];
    let vertical = |w, opposite| Arm {
        w,
        opposite,
        sides: [a.left, a.right],
        own: columns,
        across: rows,
        len: FONT_H,
    };
    let horizontal = |w, opposite| Arm {
        w,
        opposite,
        sides: [a.up, a.down],
        own: rows,
        across: columns,
        len: FONT_W,
    };
    if a.up != W::N {
        for ((x0, x1), end) in vertical(a.up, a.down).near_edge_rails() {
            fill_rect(&mut mask, x0, 0, x1, end);
        }
    }
    if a.down != W::N {
        for ((x0, x1), start) in vertical(a.down, a.up).far_edge_rails() {
            fill_rect(&mut mask, x0, start, x1, FONT_H);
        }
    }
    if a.left != W::N {
        for ((y0, y1), end) in horizontal(a.left, a.right).near_edge_rails() {
            fill_rect(&mut mask, 0, y0, end, y1);
        }
    }
    if a.right != W::N {
        for ((y0, y1), start) in horizontal(a.right, a.left).far_edge_rails() {
            fill_rect(&mut mask, start, y0, FONT_W, y1);
        }
    }
    mask
}

// ----------------------------------------------------------------------------
// Block elements (U+2580-U+259F)
// ----------------------------------------------------------------------------

/// The block elements: halves, eighths, quadrants and shades.
fn block(ch: char) -> Option<GlyphMask> {
    let mut m = [0u8; FONT_H];
    let (w, h) = (FONT_W, FONT_H);
    let (hw, hh) = (w / 2, h / 2);
    let cp = ch as u32;
    match ch {
        '▀' => fill_rect(&mut m, 0, 0, w, hh),
        // ▁▂▃▄▅▆▇█: the lower n eighths.
        '\u{2581}'..='\u{2588}' => {
            let eighths = (cp - 0x2580) as usize;
            fill_rect(&mut m, 0, h - eighths * h / 8, w, h);
        }
        // ▉▊▋▌▍▎▏: the left n eighths, seven down to one.
        '\u{2589}'..='\u{258F}' => {
            let eighths = (0x2590 - cp) as usize;
            fill_rect(&mut m, 0, 0, eighths * w / 8, h);
        }
        '▐' => fill_rect(&mut m, hw, 0, w, h),
        '░' => return Some(shade(|x, y| x % 2 == 0 && y % 2 == 0)),
        '▒' => return Some(shade(|x, y| (x + y) % 2 == 0)),
        '▓' => return Some(shade(|x, y| !(x % 2 == 1 && y % 2 == 1))),
        '▔' => fill_rect(&mut m, 0, 0, w, h / 8),
        '▕' => fill_rect(&mut m, w - w / 8, 0, w, h),
        // Quadrants: upper-left, upper-right, lower-left, lower-right.
        '\u{2596}'..='\u{259F}' => {
            let [ul, ur, ll, lr] = match ch {
                '▖' => [false, false, true, false],
                '▗' => [false, false, false, true],
                '▘' => [true, false, false, false],
                '▙' => [true, false, true, true],
                '▚' => [true, false, false, true],
                '▛' => [true, true, true, false],
                '▜' => [true, true, false, true],
                '▝' => [false, true, false, false],
                '▞' => [false, true, true, false],
                _ => [false, true, true, true], // ▟
            };
            for (on, x, y) in [(ul, 0, 0), (ur, hw, 0), (ll, 0, hh), (lr, hw, hh)] {
                if on {
                    fill_rect(&mut m, x, y, x + hw, y + hh);
                }
            }
        }
        _ => return None,
    }
    Some(m)
}

/// Build a shade glyph from a per-pixel predicate.
fn shade(on: impl Fn(usize, usize) -> bool) -> GlyphMask {
    let mut mask = [0u8; FONT_H];
    for y in 0..FONT_H {
        for x in 0..FONT_W {
            if on(x, y) {
                set_px(&mut mask, x, y);
            }
        }
    }
    mask
}

// ----------------------------------------------------------------------------
// Triangles
// ----------------------------------------------------------------------------

/// Which corner a diagonal half-cell triangle points into.
#[derive(Clone, Copy)]
enum Corner {
    LowerRight,
    LowerLeft,
    UpperLeft,
    UpperRight,
}

/// Build a right-triangle filling one diagonal half of the cell.
fn corner_triangle(corner: Corner) -> GlyphMask {
    let (w1, h1) = (FONT_W - 1, FONT_H - 1);
    shade(|x, y| {
        // Normalize coordinates to the diagonal test; pick the half-plane
        // whose right angle sits in the requested corner.
        let (dx, dy) = match corner {
            Corner::LowerRight => (x, h1 - y),
            Corner::LowerLeft => (w1 - x, h1 - y),
            Corner::UpperLeft => (w1 - x, y),
            Corner::UpperRight => (x, y),
        };
        dx * h1 >= dy * w1
    })
}

/// Direction for arrow glyphs.
#[derive(Clone, Copy)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

/// Build a filled triangle pointing in the given direction.
fn triangle(dir: Dir) -> GlyphMask {
    let mut mask = [0u8; FONT_H];
    match dir {
        // Vertical arrows: widen one row per step over the middle of the cell.
        Dir::Up => {
            for (i, y) in (4..12).enumerate() {
                let half = i / 2 + 1;
                let cx = FONT_W / 2;
                fill_rect(
                    &mut mask,
                    cx.saturating_sub(half),
                    y,
                    (cx + half).min(FONT_W),
                    y + 1,
                );
            }
        }
        Dir::Down => {
            for (i, y) in (4..12).enumerate() {
                let half = (8 - i) / 2 + 1;
                let cx = FONT_W / 2;
                fill_rect(
                    &mut mask,
                    cx.saturating_sub(half),
                    y,
                    (cx + half).min(FONT_W),
                    y + 1,
                );
            }
        }
        // Horizontal arrows: widen one column per step.
        Dir::Left => {
            for (i, x) in (1..7).enumerate() {
                let half = i / 2 + 1;
                let cy = FONT_H / 2;
                fill_rect(
                    &mut mask,
                    x,
                    cy.saturating_sub(half),
                    x + 1,
                    (cy + half).min(FONT_H),
                );
            }
        }
        Dir::Right => {
            for (i, x) in (1..7).enumerate() {
                let half = (6 - i) / 2 + 1;
                let cy = FONT_H / 2;
                fill_rect(
                    &mut mask,
                    x,
                    cy.saturating_sub(half),
                    x + 1,
                    (cy + half).min(FONT_H),
                );
            }
        }
    }
    mask
}
