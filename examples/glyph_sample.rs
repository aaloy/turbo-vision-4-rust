// (C) 2026 - Enzo Lombardi
// Glyph Sample
//
// Renders every character the PNG screenshot renderer draws beyond ASCII into
// `target/glyph-sample.png`, at 2x: the framework's own marks, CP437's
// symbols, box drawing (light, heavy, double, dashed, rounded and diagonal)
// and blocks, and the Latin-1 Supplement, with a few frames that mix line
// weights to show how they tile.
//
// Run with:
//   cargo run --example glyph_sample

use std::path::Path;
use turbo_vision::core::draw::Cell;
use turbo_vision::core::palette::{Attr, TvColor};
use turbo_vision::core::screenshot::render_to_png;

const LINES: &[&str] = &[
    " Framework: • ℹ → √ ◆ ✓ ⚠ ❌ ❓ ■ ▲ ▼ ◄ ► ◢ ◣ ◤ ◥",
    " Progress:  █▉▊▋▌▍▎▏ ▏▎▍▌▋▊▉█ ░░▒▒▓▓",
    " CP437:     ☺☻♥♦♣♠•◘○◙♂♀♪♫☼►◄↕‼¶§▬↨↑↓→←∟↔▲▼⌂",
    "            ¢£¥₧ƒªº¿⌐¬½¼¡«» αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■",
    " Blocks:    ▀▁▂▃▄▅▆▇█▉▊▋▌▍▎▏▐░▒▓▔▕▖▗▘▙▚▛▜▝▞▟",
    " Boxes:     ─│┌┐└┘├┤┬┴┼═║╒╓╔╕╖╗╘╙╚╛╜╝╞╟╠╡╢╣╤╥╦╧╨╩╪╫╬",
    " Heavy:     ━┃┏┓┗┛┣┫┳┻╋ ┍┎┑┒┕┖┙┚┝┞┟┠┡┢┥┦┧┨┩┪┭┮┯┰┱┲┵┶┷┸┹┺┽┾┿╀╁╂╃╄╅╆╇╈╉╊",
    " Dashed:    ┄┅┆┇┈┉┊┋╌╍╎╏  Rounded: ╭╮╯╰  Diagonal: ╱╲╳  Half: ╴╵╶╷╸╹╺╻╼╽╾╿",
    " Latin-1:   ¡¢£¤¥¦§¨©ª«¬®¯°±²³´µ¶·¸¹º»¼½¾¿",
    "            ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞß",
    "            àáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþÿ",
    " Text:      Café, naïve, Ærøskøbing, Größe, ¿Qué?, ¡Olé!, 25°C ±1, x² ≤ y",
    "",
    " ┌───┬───┐  ╔═══╦═══╗  ╒═══╤═══╕  ╓───╥───╖  ┌─────┐",
    " │   │   │  ║   ║   ║  │   │   │  ║   ║   ║  │ ⌠   │",
    " ├───┼───┤  ╠═══╬═══╣  ╞═══╪═══╡  ╟───╫───╢  │ │ ∞ │",
    " │   │   │  ║   ║   ║  │   │   │  ║   ║   ║  │ ⌡   │",
    " └───┴───┘  ╚═══╩═══╝  ╘═══╧═══╛  ╙───╨───╜  └─────┘",
    "",
    " Heavy/dashed/rounded:",
    " ┏━━━┳━━━┓  ┍━━━┯━━━┑  ┎───┰───┒  ╭───┬───╮  ┌┄┄┄┬╌╌╌┐  ╱╲╱╲  ╶━━╴",
    " ┃   ┃   ┃  │   │   │  ┃   ┃   ┃  │   │   │  ┆   ┇   ╎  ╲╱╲╱  ╻  ╷",
    " ┣━━━╋━━━┫  ┝━━━┿━━━┥  ┠───╂───┨  ├───┼───┤  ├┈┈┈┼┉┉┉┤  ╳╳╳╳  ┃  │",
    " ┃   ┃   ┃  │   │   │  ┃   ┃   ┃  │   │   │  ┊   ┋   ╏  ╱╲╱╲  ╹  ╵",
    " ┗━━━┻━━━┛  ┕━━━┷━━━┙  ┖───┸───┚  ╰───┴───╯  └╍╍╍┴┅┅┅┘  ╲╱╲╱  ╺──╸",
];

fn main() -> std::io::Result<()> {
    let attr = Attr::new(TvColor::White, TvColor::Blue);
    let cols = LINES.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let buffer: Vec<Vec<Cell>> = LINES
        .iter()
        .map(|line| {
            let mut row: Vec<Cell> = line.chars().map(|c| Cell::new(c, attr)).collect();
            row.resize(cols, Cell::new(' ', attr));
            row
        })
        .collect();
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/glyph-sample.png");
    render_to_png(&buffer, cols, buffer.len(), 2, &path)?;
    println!("{}", path.display());
    Ok(())
}
