// (C) 2025 - Enzo Lombardi

//! Key types, from one place.
//!
//! With the `native` feature these are crossterm's own, so nothing changes for
//! a terminal application. Without it they are local look-alikes with the
//! same names, so views compile for an embedder that has no terminal, such as
//! a WASM guest that is handed keys by its host.

#[cfg(feature = "native")]
pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[cfg(not(feature = "native"))]
mod local {
    use std::ops::{BitOr, BitOrAssign};

    /// A key, as crossterm names them. Only the variants Turbo Vision reads.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum KeyCode {
        Backspace,
        Enter,
        Left,
        Right,
        Up,
        Down,
        Home,
        End,
        PageUp,
        PageDown,
        Tab,
        BackTab,
        Delete,
        Insert,
        F(u8),
        Char(char),
        Null,
        Esc,
    }

    /// Modifier bits, spelled as crossterm's bitflags spell them.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct KeyModifiers(u8);

    impl KeyModifiers {
        pub const NONE: Self = Self(0);
        pub const SHIFT: Self = Self(0b0001);
        pub const CONTROL: Self = Self(0b0010);
        pub const ALT: Self = Self(0b0100);

        #[must_use]
        pub const fn empty() -> Self {
            Self(0)
        }

        #[must_use]
        pub const fn contains(self, other: Self) -> bool {
            self.0 & other.0 == other.0
        }

        #[must_use]
        pub const fn is_empty(self) -> bool {
            self.0 == 0
        }

        pub fn insert(&mut self, other: Self) {
            self.0 |= other.0;
        }

        pub fn remove(&mut self, other: Self) {
            self.0 &= !other.0;
        }
    }

    impl BitOr for KeyModifiers {
        type Output = Self;
        fn bitor(self, rhs: Self) -> Self {
            Self(self.0 | rhs.0)
        }
    }

    impl BitOrAssign for KeyModifiers {
        fn bitor_assign(&mut self, rhs: Self) {
            self.0 |= rhs.0;
        }
    }

    /// A key press.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct KeyEvent {
        pub code: KeyCode,
        pub modifiers: KeyModifiers,
    }

    impl KeyEvent {
        #[must_use]
        pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
            Self { code, modifiers }
        }
    }
}

#[cfg(not(feature = "native"))]
pub use local::{KeyCode, KeyEvent, KeyModifiers};
