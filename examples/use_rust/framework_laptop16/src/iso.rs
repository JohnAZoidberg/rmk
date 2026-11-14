// Copyright 2022–2023 Framework Computer
// SPDX-License-Identifier: GPL-2.0-or-later
// Matrix positions and default keymaps derived from Framework QMK firmware.
use rmk::{k, types::action::KeyAction};
pub const ROW: usize = 8;
pub const COL: usize = 16;
pub const PID: u16 = 0x0012;
#[rustfmt::skip]
pub const fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; 4] {
    [
        [
            [k!(C), k!(Delete), k!(Q), k!(RAlt), KeyAction::No, k!(X), k!(V), k!(M), k!(Dot), k!(RShift), k!(Comma), KeyAction::No, k!(RCtrl), k!(Slash), k!(Quote), KeyAction::No],
            [KeyAction::No, KeyAction::No, KeyAction::No, k!(LAlt), k!(Space), k!(Z), k!(B), k!(N), k!(Down), k!(LShift), KeyAction::No, KeyAction::No, k!(LCtrl), k!(Up), k!(Enter), k!(BrightnessUp)],
            [KeyAction::No, KeyAction::No, rmk::mo!(1), KeyAction::No, k!(E), k!(AudioVolDown), k!(G), k!(H), k!(NonusHash), KeyAction::No, k!(BrightnessDown), KeyAction::No, KeyAction::No, k!(Minus), KeyAction::No, k!(Right)],
            [KeyAction::No, k!(LGui), k!(Tab), KeyAction::No, k!(MediaPrevTrack), k!(AudioMute), k!(T), k!(Y), k!(O), KeyAction::No, k!(MediaNextTrack), rmk::wm!(P, rmk::types::modifier::ModifierCombination::LGUI), KeyAction::No, k!(MediaSelect), KeyAction::No, KeyAction::No],
            [KeyAction::No, KeyAction::No, k!(Grave), KeyAction::No, k!(CapsLock), k!(S), k!(Kc5), k!(Kc6), KeyAction::No, KeyAction::No, k!(MediaPlayPause), KeyAction::No, KeyAction::No, k!(Kc0), k!(Equal), KeyAction::No],
            [KeyAction::No, KeyAction::No, k!(Kc1), KeyAction::No, k!(Kc3), k!(Kc2), k!(Kc4), k!(Kc7), k!(Kc9), KeyAction::No, k!(Kc8), k!(NonusBackslash), KeyAction::No, k!(P), k!(Backspace), KeyAction::No],
            [KeyAction::No, KeyAction::No, KeyAction::No, KeyAction::No, k!(AudioVolUp), k!(W), k!(R), k!(U), k!(PrintScreen), KeyAction::No, k!(I), k!(Left), KeyAction::No, k!(LeftBracket), k!(RightBracket), KeyAction::No],
            [KeyAction::No, KeyAction::No, k!(A), KeyAction::No, KeyAction::No, k!(Escape), k!(F), k!(J), k!(L), KeyAction::No, k!(K), KeyAction::No, KeyAction::No, k!(Semicolon), k!(D), KeyAction::No],
        ],
        [
            [KeyAction::No, k!(Insert), KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No],
            [KeyAction::No, KeyAction::No, KeyAction::No, KeyAction::Transparent, KeyAction::No, KeyAction::Transparent, k!(Pause), KeyAction::Transparent, k!(PageDown), KeyAction::Transparent, KeyAction::No, KeyAction::No, KeyAction::Transparent, k!(PageUp), KeyAction::Transparent, k!(F8)],
            [KeyAction::No, KeyAction::No, KeyAction::Transparent, KeyAction::No, KeyAction::Transparent, k!(F2), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, k!(F7), KeyAction::No, KeyAction::No, KeyAction::Transparent, KeyAction::No, k!(End)],
            [KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, k!(F4), k!(F1), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, k!(F6), k!(F9), KeyAction::No, k!(F12), KeyAction::No, KeyAction::No],
            [KeyAction::No, KeyAction::No, KeyAction::Transparent, KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, k!(F10), KeyAction::No, k!(F5), KeyAction::No, KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No],
            [KeyAction::No, KeyAction::No, KeyAction::Transparent, KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, k!(Pause), KeyAction::Transparent, KeyAction::No],
            [KeyAction::No, KeyAction::No, KeyAction::No, KeyAction::No, k!(F3), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, k!(F11), KeyAction::No, KeyAction::Transparent, k!(Home), KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No],
            [KeyAction::No, KeyAction::No, KeyAction::Transparent, KeyAction::No, KeyAction::No, rmk::pdf!(2), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, k!(ScrollLock), KeyAction::No, KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No],
        ],
        [
            [k!(C), k!(Delete), k!(Q), k!(RAlt), KeyAction::No, k!(X), k!(V), k!(M), k!(Dot), k!(RShift), k!(Comma), KeyAction::No, k!(RCtrl), k!(Slash), k!(Quote), KeyAction::No],
            [KeyAction::No, KeyAction::No, KeyAction::No, k!(LAlt), k!(Space), k!(Z), k!(B), k!(N), k!(Down), k!(LShift), KeyAction::No, KeyAction::No, k!(LCtrl), k!(Up), k!(Enter), k!(F8)],
            [KeyAction::No, KeyAction::No, rmk::mo!(3), KeyAction::No, k!(E), k!(F2), k!(G), k!(H), k!(NonusHash), KeyAction::No, k!(F7), KeyAction::No, KeyAction::No, k!(Minus), KeyAction::No, k!(Right)],
            [KeyAction::No, k!(LGui), k!(Tab), KeyAction::No, k!(F4), k!(F1), k!(T), k!(Y), k!(O), KeyAction::No, k!(F6), k!(F9), KeyAction::No, k!(F12), KeyAction::No, KeyAction::No],
            [KeyAction::No, KeyAction::No, k!(Grave), KeyAction::No, k!(CapsLock), k!(S), k!(Kc5), k!(Kc6), k!(F10), KeyAction::No, k!(F5), KeyAction::No, KeyAction::No, k!(Kc0), k!(Equal), KeyAction::No],
            [KeyAction::No, KeyAction::No, k!(Kc1), KeyAction::No, k!(Kc3), k!(Kc2), k!(Kc4), k!(Kc7), k!(Kc9), KeyAction::No, k!(Kc8), k!(NonusBackslash), KeyAction::No, k!(P), k!(Backspace), KeyAction::No],
            [KeyAction::No, KeyAction::No, KeyAction::No, KeyAction::No, k!(F3), k!(W), k!(R), k!(U), k!(F11), KeyAction::No, k!(I), k!(Left), KeyAction::No, k!(LeftBracket), k!(RightBracket), KeyAction::No],
            [KeyAction::No, KeyAction::No, k!(A), KeyAction::No, KeyAction::No, k!(Escape), k!(F), k!(J), k!(L), KeyAction::No, k!(K), KeyAction::No, KeyAction::No, k!(Semicolon), k!(D), KeyAction::No],
        ],
        [
            [KeyAction::No, k!(Insert), KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No],
            [KeyAction::No, KeyAction::No, KeyAction::No, KeyAction::Transparent, KeyAction::No, KeyAction::Transparent, k!(Pause), KeyAction::Transparent, k!(PageDown), KeyAction::Transparent, KeyAction::No, KeyAction::No, KeyAction::Transparent, k!(PageUp), KeyAction::Transparent, k!(BrightnessUp)],
            [KeyAction::No, KeyAction::No, KeyAction::Transparent, KeyAction::No, KeyAction::Transparent, k!(AudioVolDown), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, k!(BrightnessDown), KeyAction::No, KeyAction::No, KeyAction::Transparent, KeyAction::No, k!(End)],
            [KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, k!(MediaPrevTrack), k!(AudioMute), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, k!(MediaNextTrack), rmk::wm!(P, rmk::types::modifier::ModifierCombination::LGUI), KeyAction::No, k!(MediaSelect), KeyAction::No, KeyAction::No],
            [KeyAction::No, KeyAction::No, KeyAction::Transparent, KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, KeyAction::No, k!(MediaPlayPause), KeyAction::No, KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No],
            [KeyAction::No, KeyAction::No, KeyAction::Transparent, KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, k!(Pause), KeyAction::Transparent, KeyAction::No],
            [KeyAction::No, KeyAction::No, KeyAction::No, KeyAction::No, k!(AudioVolUp), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, k!(PrintScreen), KeyAction::No, KeyAction::Transparent, k!(Home), KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No],
            [KeyAction::No, KeyAction::No, KeyAction::Transparent, KeyAction::No, KeyAction::No, rmk::pdf!(0), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No, k!(ScrollLock), KeyAction::No, KeyAction::No, KeyAction::Transparent, KeyAction::Transparent, KeyAction::No],
        ],
    ]
}
