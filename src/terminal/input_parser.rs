// (C) 2025 - Enzo Lombardi

//! Input parser for raw terminal byte streams.
//!
//! This module provides the [`InputParser`] type which converts raw terminal
//! input bytes (ANSI escape sequences) into turbo-vision [`Event`] structures.
//! This is primarily used by remote backends, such as the `tv-extensions`
//! crate's SSH backend, to parse input from remote terminal clients.
//!
//! # Supported Input
//!
//! - Regular ASCII characters
//! - UTF-8 multi-byte characters
//! - Control characters (Ctrl+A through Ctrl+Z)
//! - Function keys (F1-F12)
//! - Arrow keys and navigation keys
//! - Mouse events (X10 and SGR formats)
//! - Modifier combinations (Shift, Alt, Ctrl)

use crate::core::event::{
    Event, EventType, KB_ALT_A, KB_ALT_B, KB_ALT_C, KB_ALT_D, KB_ALT_E, KB_ALT_F, KB_ALT_G,
    KB_ALT_H, KB_ALT_I, KB_ALT_J, KB_ALT_K, KB_ALT_L, KB_ALT_M, KB_ALT_N, KB_ALT_O, KB_ALT_P,
    KB_ALT_Q, KB_ALT_R, KB_ALT_S, KB_ALT_T, KB_ALT_U, KB_ALT_V, KB_ALT_W, KB_ALT_X, KB_ALT_Y,
    KB_ALT_Z, KB_BACKSPACE, KB_ENTER, KB_ESC, KB_SHIFT_TAB, KB_TAB, MB_LEFT_BUTTON,
    MB_MIDDLE_BUTTON, MB_RIGHT_BUTTON,
};
use crate::core::geometry::Point;
use crate::core::keys::{KeyCode as CKC, KeyEvent, KeyModifiers};

/// Parser for raw terminal input bytes.
///
/// Maintains an internal buffer to handle multi-byte sequences and
/// incomplete escape sequences that may arrive across multiple reads.
///
/// # Example
///
/// ```rust
/// use turbo_vision::terminal::InputParser;
///
/// let mut parser = InputParser::new();
///
/// // Feed raw bytes and extract events
/// let events = parser.parse(b"\x1b[A"); // Up arrow
/// assert_eq!(events.len(), 1);
/// ```
pub struct InputParser {
    buffer: Vec<u8>,
}

/// Longest escape sequence the parser will buffer while waiting for more
/// bytes. A malformed sequence with no final byte would otherwise grow the
/// buffer without bound on hostile input (e.g. from a remote client, such as
/// the `tv-extensions` crate's SSH backend serves).
const MAX_PENDING_SEQUENCE: usize = 64;

impl InputParser {
    /// Create a new input parser.
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(64),
        }
    }

    /// Feed raw bytes and extract events.
    ///
    /// Returns a vector of events parsed from the input. Incomplete
    /// sequences are buffered for the next call.
    pub fn parse(&mut self, data: &[u8]) -> Vec<Event> {
        self.buffer.extend_from_slice(data);
        let mut events = Vec::new();

        while !self.buffer.is_empty() {
            match self.try_parse() {
                Some((event, consumed)) => {
                    events.push(event);
                    self.buffer.drain(..consumed);
                }
                None => {
                    // Incomplete sequence: wait for more data — unless it has
                    // already exceeded any legitimate sequence length, in
                    // which case drop the leading byte to resynchronize
                    if self.buffer.len() > MAX_PENDING_SEQUENCE {
                        self.buffer.remove(0);
                        continue;
                    }
                    break;
                }
            }
        }
        events
    }

    /// Clear the internal buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Try to parse a single event from the buffer.
    ///
    /// Returns `Some((event, bytes_consumed))` if successful, `None` if
    /// more data is needed.
    fn try_parse(&self) -> Option<(Event, usize)> {
        let buf = &self.buffer;
        if buf.is_empty() {
            return None;
        }

        // ESC sequences
        if buf[0] == 0x1b {
            return self.parse_escape_sequence();
        }

        // Control characters and regular input
        match buf[0] {
            0x0d => Some((Event::keyboard(KB_ENTER), 1)),
            0x09 => Some((Event::keyboard(KB_TAB), 1)),
            0x7f | 0x08 => Some((Event::keyboard(KB_BACKSPACE), 1)),
            0x01..=0x1a => {
                // Control characters (Ctrl+A = 0x01, Ctrl+B = 0x02, etc.)
                let ctrl_code = buf[0] as u16;
                Some((Event::keyboard(ctrl_code), 1))
            }
            c if c >= 0x20 => self.parse_utf8(),
            _ => Some((Event::keyboard(0), 1)), // Null/unknown
        }
    }

    /// Parse an escape sequence starting with ESC (0x1b).
    fn parse_escape_sequence(&self) -> Option<(Event, usize)> {
        let buf = &self.buffer;
        if buf.len() < 2 {
            return None; // Need more data
        }

        match buf[1] {
            b'[' => self.parse_csi(),
            b'O' => self.parse_ss3(),
            c if c.is_ascii_alphabetic() => {
                // ESC + letter = Alt+letter
                if let Some(alt_code) = char_to_alt_code((c as char).to_ascii_lowercase()) {
                    Some((Event::keyboard(alt_code), 2))
                } else {
                    Some((Event::keyboard(KB_ESC), 1))
                }
            }
            _ => Some((Event::keyboard(KB_ESC), 1)),
        }
    }

    /// Parse CSI (Control Sequence Introducer) sequences: ESC [
    fn parse_csi(&self) -> Option<(Event, usize)> {
        let buf = &self.buffer;
        if buf.len() < 3 {
            return None;
        }

        // Check for mouse sequences first
        if buf[2] == b'<' {
            return self.parse_mouse_sgr();
        }
        if buf[2] == b'M' {
            // X10 mouse: ESC [ M b x y (6 bytes). A partial sequence must
            // wait for the rest — falling through would treat 'M' as a CSI
            // final byte and desynchronize the stream
            if buf.len() < 6 {
                return None;
            }
            return self.parse_mouse_normal();
        }

        // Find final byte (0x40..=0x7E)
        let end = buf[2..]
            .iter()
            .position(|&b| (0x40..=0x7E).contains(&b))
            .map(|i| i + 3)?;

        let params = &buf[2..end - 1];
        let final_byte = buf[end - 1];

        let code = match final_byte {
            b'A' => Some(CKC::Up),
            b'B' => Some(CKC::Down),
            b'C' => Some(CKC::Right),
            b'D' => Some(CKC::Left),
            b'H' => Some(CKC::Home),
            b'F' => Some(CKC::End),
            // F1-F4 with modifiers: ESC [ 1 ; m P..S
            b'P' => Some(CKC::F(1)),
            b'Q' => Some(CKC::F(2)),
            b'R' => Some(CKC::F(3)),
            b'S' => Some(CKC::F(4)),
            b'Z' => return Some((Event::keyboard(KB_SHIFT_TAB), end)),
            b'~' => tilde_key(params),
            _ => None,
        };

        Some((key_event(code, csi_modifiers(params)), end))
    }

    /// Parse SS3 (Single Shift 3) sequences: ESC O
    fn parse_ss3(&self) -> Option<(Event, usize)> {
        if self.buffer.len() < 3 {
            return None;
        }

        let code = match self.buffer[2] {
            b'P' => Some(CKC::F(1)),
            b'Q' => Some(CKC::F(2)),
            b'R' => Some(CKC::F(3)),
            b'S' => Some(CKC::F(4)),
            b'A' => Some(CKC::Up),
            b'B' => Some(CKC::Down),
            b'C' => Some(CKC::Right),
            b'D' => Some(CKC::Left),
            b'H' => Some(CKC::Home),
            b'F' => Some(CKC::End),
            _ => None,
        };

        Some((key_event(code, KeyModifiers::empty()), 3))
    }

    /// Parse a UTF-8 character.
    fn parse_utf8(&self) -> Option<(Event, usize)> {
        if let Ok(s) = std::str::from_utf8(&self.buffer) {
            if let Some(ch) = s.chars().next() {
                return Some((Event::text(ch), ch.len_utf8()));
            }
        }
        if self.buffer.len() < 4 {
            None // Incomplete UTF-8
        } else {
            Some((Event::keyboard(0), 1)) // Invalid - skip byte
        }
    }

    /// Parse X10 mouse protocol: ESC [ M Cb Cx Cy
    fn parse_mouse_normal(&self) -> Option<(Event, usize)> {
        if self.buffer.len() < 6 {
            return None;
        }

        let cb = self.buffer[3].wrapping_sub(32);
        let cx = self.buffer[4].wrapping_sub(32).saturating_sub(1) as i16;
        let cy = self.buffer[5].wrapping_sub(32).saturating_sub(1) as i16;
        let pos = Point::new(cx, cy);

        let event = if cb & 0x40 != 0 {
            // Scroll wheel
            let event_type = if cb & 0x01 != 0 {
                EventType::MouseWheelDown
            } else {
                EventType::MouseWheelUp
            };
            Event::mouse(event_type, pos, 0, false)
        } else if cb & 0x03 == 3 {
            // Button release
            Event::mouse(EventType::MouseUp, pos, 0, false)
        } else {
            let button = match cb & 0x03 {
                0 => MB_LEFT_BUTTON,
                1 => MB_MIDDLE_BUTTON,
                2 => MB_RIGHT_BUTTON,
                _ => MB_LEFT_BUTTON,
            };
            Event::mouse(EventType::MouseDown, pos, button, false)
        };

        Some((event, 6))
    }

    /// Parse SGR mouse protocol: ESC [ < Cb ; Cx ; Cy M/m
    fn parse_mouse_sgr(&self) -> Option<(Event, usize)> {
        // Find the final M or m
        let end = self.buffer[3..]
            .iter()
            .position(|&b| b == b'M' || b == b'm')
            .map(|i| i + 4)?;

        let params = std::str::from_utf8(&self.buffer[3..end - 1]).ok()?;
        let mut parts = params.split(';');

        let cb: u8 = parts.next()?.parse().ok()?;
        let cx: i16 = parts.next()?.parse::<i16>().ok()?.saturating_sub(1);
        let cy: i16 = parts.next()?.parse::<i16>().ok()?.saturating_sub(1);
        let pressed = self.buffer[end - 1] == b'M';
        let pos = Point::new(cx, cy);

        let event = if cb & 64 != 0 {
            // Scroll wheel
            let event_type = if cb & 1 != 0 {
                EventType::MouseWheelDown
            } else {
                EventType::MouseWheelUp
            };
            Event::mouse(event_type, pos, 0, false)
        } else if cb & 32 != 0 {
            // Motion event (drag)
            let button = match cb & 0x03 {
                0 => MB_LEFT_BUTTON,
                1 => MB_MIDDLE_BUTTON,
                2 => MB_RIGHT_BUTTON,
                _ => 0,
            };
            Event::mouse(EventType::MouseMove, pos, button, false)
        } else {
            let button = match cb & 0x03 {
                0 => MB_LEFT_BUTTON,
                1 => MB_MIDDLE_BUTTON,
                2 => MB_RIGHT_BUTTON,
                _ => MB_LEFT_BUTTON,
            };
            let event_type = if pressed {
                EventType::MouseDown
            } else {
                EventType::MouseUp
            };
            Event::mouse(event_type, pos, button, false)
        };

        Some((event, end))
    }
}

impl Default for InputParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a lowercase letter to its Alt+letter key code.
fn char_to_alt_code(c: char) -> Option<u16> {
    match c {
        'a' => Some(KB_ALT_A),
        'b' => Some(KB_ALT_B),
        'c' => Some(KB_ALT_C),
        'd' => Some(KB_ALT_D),
        'e' => Some(KB_ALT_E),
        'f' => Some(KB_ALT_F),
        'g' => Some(KB_ALT_G),
        'h' => Some(KB_ALT_H),
        'i' => Some(KB_ALT_I),
        'j' => Some(KB_ALT_J),
        'k' => Some(KB_ALT_K),
        'l' => Some(KB_ALT_L),
        'm' => Some(KB_ALT_M),
        'n' => Some(KB_ALT_N),
        'o' => Some(KB_ALT_O),
        'p' => Some(KB_ALT_P),
        'q' => Some(KB_ALT_Q),
        'r' => Some(KB_ALT_R),
        's' => Some(KB_ALT_S),
        't' => Some(KB_ALT_T),
        'u' => Some(KB_ALT_U),
        'v' => Some(KB_ALT_V),
        'w' => Some(KB_ALT_W),
        'x' => Some(KB_ALT_X),
        'y' => Some(KB_ALT_Y),
        'z' => Some(KB_ALT_Z),
        _ => None,
    }
}

/// The event for a key with modifiers, made the way the crossterm backend
/// makes it, so a key from a byte stream looks exactly like the same key
/// from a local terminal. An unknown key is key code 0, as before.
fn key_event(code: Option<CKC>, modifiers: KeyModifiers) -> Event {
    match code {
        Some(code) => Event::from_crossterm_key(KeyEvent::new(code, modifiers)),
        None => Event::keyboard(0),
    }
}

/// The key of a tilde-terminated sequence: ESC [ number ( ; m ) ~
fn tilde_key(params: &[u8]) -> Option<CKC> {
    let num: u8 = params
        .iter()
        .take_while(|&&b| b.is_ascii_digit())
        .fold(0, |acc, &b| acc.saturating_mul(10).saturating_add(b - b'0'));

    Some(match num {
        1 | 7 => CKC::Home,
        2 => CKC::Insert,
        3 => CKC::Delete,
        4 | 8 => CKC::End,
        5 => CKC::PageUp,
        6 => CKC::PageDown,
        11 => CKC::F(1),
        12 => CKC::F(2),
        13 => CKC::F(3),
        14 => CKC::F(4),
        15 => CKC::F(5),
        17 => CKC::F(6),
        18 => CKC::F(7),
        19 => CKC::F(8),
        20 => CKC::F(9),
        21 => CKC::F(10),
        23 => CKC::F(11),
        24 => CKC::F(12),
        _ => return None,
    })
}

/// The modifiers in a CSI sequence's second parameter, as xterm encodes
/// them: m = 1 + (Shift=1 | Alt=2 | Ctrl=4 | Meta=8). Meta has no
/// `KeyModifiers` bit here and is ignored.
fn csi_modifiers(params: &[u8]) -> KeyModifiers {
    let m: u16 = std::str::from_utf8(params)
        .ok()
        .and_then(|s| s.split(';').nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let bits = m.saturating_sub(1);

    let mut mods = KeyModifiers::empty();
    if bits & 1 != 0 {
        mods |= KeyModifiers::SHIFT;
    }
    if bits & 2 != 0 {
        mods |= KeyModifiers::ALT;
    }
    if bits & 4 != 0 {
        mods |= KeyModifiers::CONTROL;
    }
    mods
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::{KB_DOWN, KB_F1, KB_F5, KB_LEFT, KB_RIGHT, KB_UP};

    #[test]
    fn test_parse_regular_chars() {
        let mut parser = InputParser::new();
        let events = parser.parse(b"abc");
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].key_code, 'a' as u16);
        assert_eq!(events[1].key_code, 'b' as u16);
        assert_eq!(events[2].key_code, 'c' as u16);
    }

    #[test]
    fn test_parse_arrow_keys() {
        let mut parser = InputParser::new();

        let events = parser.parse(b"\x1b[A");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].key_code, KB_UP);

        let events = parser.parse(b"\x1b[B");
        assert_eq!(events[0].key_code, KB_DOWN);

        let events = parser.parse(b"\x1b[C");
        assert_eq!(events[0].key_code, KB_RIGHT);

        let events = parser.parse(b"\x1b[D");
        assert_eq!(events[0].key_code, KB_LEFT);
    }

    #[test]
    fn test_parse_function_keys() {
        let mut parser = InputParser::new();

        let events = parser.parse(b"\x1bOP");
        assert_eq!(events[0].key_code, KB_F1);

        let events = parser.parse(b"\x1b[15~");
        assert_eq!(events[0].key_code, KB_F5);
    }

    #[test]
    fn test_parse_enter_and_backspace() {
        let mut parser = InputParser::new();

        let events = parser.parse(b"\r");
        assert_eq!(events[0].key_code, KB_ENTER);

        let events = parser.parse(b"\x7f");
        assert_eq!(events[0].key_code, KB_BACKSPACE);
    }

    #[test]
    fn test_parse_control_chars() {
        let mut parser = InputParser::new();

        // Ctrl+A = 0x01
        let events = parser.parse(b"\x01");
        assert_eq!(events[0].key_code, 0x01);

        // Ctrl+C = 0x03
        let events = parser.parse(b"\x03");
        assert_eq!(events[0].key_code, 0x03);
    }

    #[test]
    fn test_parse_alt_letters() {
        let mut parser = InputParser::new();

        let events = parser.parse(b"\x1bx");
        assert_eq!(events[0].key_code, KB_ALT_X);

        let events = parser.parse(b"\x1bf");
        assert_eq!(events[0].key_code, KB_ALT_F);
    }

    #[test]
    fn test_parse_mouse_sgr() {
        let mut parser = InputParser::new();

        // Left button down at (10, 5)
        let events = parser.parse(b"\x1b[<0;11;6M");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].what, EventType::MouseDown);
        assert_eq!(events[0].mouse.pos.x, 10);
        assert_eq!(events[0].mouse.pos.y, 5);
        assert_eq!(events[0].mouse.buttons, MB_LEFT_BUTTON);
    }

    #[test]
    fn test_incomplete_sequence() {
        let mut parser = InputParser::new();

        // Incomplete escape sequence
        let events = parser.parse(b"\x1b[");
        assert_eq!(events.len(), 0);

        // Complete it
        let events = parser.parse(b"A");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].key_code, KB_UP);
    }

    /// What crossterm's path makes of a key: the event the native backend
    /// would deliver.
    fn crossterm_event(code: CKC, mods: KeyModifiers) -> (u16, KeyModifiers) {
        let ev = Event::from_crossterm_key(KeyEvent::new(code, mods));
        (ev.key_code, ev.key_modifiers)
    }

    #[test]
    fn modified_keys_match_the_crossterm_path() {
        let ctrl = KeyModifiers::CONTROL;
        let shift = KeyModifiers::SHIFT;
        let alt = KeyModifiers::ALT;
        let none = KeyModifiers::empty();
        let cases: &[(&[u8], CKC, KeyModifiers)] = &[
            (b"\x1b[15;5~", CKC::F(5), ctrl),
            (b"\x1b[17;2~", CKC::F(6), shift),
            (b"\x1b[1;3C", CKC::Right, alt),
            (b"\x1b[1;5P", CKC::F(1), ctrl),
            (b"\x1b[1;3P", CKC::F(1), alt),
            (b"\x1b[1;2S", CKC::F(4), shift),
            (b"\x1b[24;5~", CKC::F(12), ctrl),
            (b"\x1b[2;5~", CKC::Insert, ctrl),
            (b"\x1b[3;2~", CKC::Delete, shift),
            (b"\x1b[5;7~", CKC::PageUp, ctrl | alt),
            (b"\x1b[1;6H", CKC::Home, ctrl | shift),
            (b"\x1b[1;5F", CKC::End, ctrl),
            (b"\x1b[1;8A", CKC::Up, ctrl | alt | shift),
            // Unmodified sequences, as before.
            (b"\x1b[15~", CKC::F(5), none),
            (b"\x1bOP", CKC::F(1), none),
            (b"\x1b[C", CKC::Right, none),
            (b"\x1b[1;1D", CKC::Left, none),
        ];
        for (bytes, code, mods) in cases {
            let mut parser = InputParser::new();
            let events = parser.parse(bytes);
            assert_eq!(events.len(), 1, "{bytes:?}");
            assert_eq!(events[0].what, EventType::Keyboard, "{bytes:?}");
            assert_eq!(
                (events[0].key_code, events[0].key_modifiers),
                crossterm_event(*code, *mods),
                "{bytes:?}"
            );
        }
        // Spot-check the codes themselves.
        let mut parser = InputParser::new();
        let ev = &parser.parse(b"\x1b[15;5~")[0];
        assert_eq!((ev.key_code, ev.key_modifiers), (KB_F5, ctrl));
        let ev = &parser.parse(b"\x1b[24;5~")[0];
        assert_eq!(ev.key_code, crate::core::event::KB_CTRL_F12);
        let ev = &parser.parse(b"\x1b[1;3P")[0];
        assert_eq!(ev.key_code, crate::core::event::KB_ALT_F1);
    }

    #[test]
    fn a_modified_key_split_across_reads_is_buffered() {
        let mut parser = InputParser::new();
        assert!(parser.parse(b"\x1b[15;").is_empty());
        let events = parser.parse(b"5~a");
        assert_eq!(events.len(), 2);
        assert_eq!(
            (events[0].key_code, events[0].key_modifiers),
            crossterm_event(CKC::F(5), KeyModifiers::CONTROL)
        );
        assert_eq!(events[1].key_code, 'a' as u16);
    }

    #[test]
    fn partial_x10_mouse_waits_for_full_sequence() {
        let mut parser = InputParser::new();
        // First 4 bytes of a 6-byte X10 mouse sequence
        let events = parser.parse(b"\x1b[M\x20");
        assert_eq!(events.len(), 0);
        // Completing it produces exactly one mouse event, no desync
        let events = parser.parse(b"\x21\x21");
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn unterminated_csi_does_not_grow_buffer_forever() {
        let mut parser = InputParser::new();
        // CSI with parameter bytes but never a final byte
        let mut junk = vec![0x1b, b'['];
        junk.extend(std::iter::repeat(b';').take(500));
        let _ = parser.parse(&junk);
        assert!(parser.buffer.len() <= 65);
        // Parser recovers: a normal key still comes through
        let events = parser.parse(b"a");
        assert!(events.iter().any(|e| e.key_code == 'a' as u16));
    }

    #[test]
    fn utf8_text_keeps_its_character_without_colliding() {
        let mut parser = InputParser::new();
        let events = parser.parse("éěł€".as_bytes());
        let typed: Vec<_> = events.iter().map(Event::typed_char).collect();
        assert_eq!(typed, [Some('é'), Some('ě'), Some('ł'), Some('€')]);
        assert_eq!(events[0].key_code, 0xE9);
        assert_eq!(
            events[1].key_code,
            crate::core::event::KB_TEXT,
            "not KB_ESC"
        );
    }
}
