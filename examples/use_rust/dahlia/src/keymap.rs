// Copyright (c) 2025 Framework Computer Inc
// SPDX-License-Identifier: MIT
// Framework 12 ANSI matrix positions and key layers, ported from ZMK.

use rmk::types::action::KeyAction;
use rmk::types::modifier::ModifierCombination;
use rmk::{a, k, mo, tg, wm};

#[rustfmt::skip]
pub const POSITIONS: [(usize, usize); 78] = [
    (1, 1), (0, 2), (3, 2), (2, 2), (1, 2), (4, 4), (2, 4), (1, 4), (2, 11), (1, 9), (0, 4), (0, 1), (1, 5), (2, 9),
    (3, 1), (5, 1), (6, 4), (6, 2), (6, 3), (3, 3), (3, 8), (6, 8), (6, 5), (6, 9), (6, 6), (3, 6), (0, 8), (1, 11),
    (2, 1), (7, 5), (7, 6), (7, 8), (7, 9), (2, 3), (2, 6), (7, 1), (7, 2), (7, 3), (7, 4), (2, 8), (2, 5), (3, 11),
    (6, 15), (4, 1), (3, 4), (4, 2), (4, 3), (1, 3), (1, 6), (4, 6), (4, 5), (4, 9), (4, 8), (1, 8), (4, 11),
    (5, 7), (6, 1), (5, 8), (5, 5), (5, 9), (0, 3), (0, 5), (5, 11), (5, 2), (5, 3), (5, 4), (7, 7),
    (1, 14), (0, 16), (3, 0), (6, 13), (5, 6), (0, 10), (3, 14), (7, 12), (7, 11), (6, 12),
    (6, 11),
];

#[rustfmt::skip]
const LAYERS: [[KeyAction; 78]; 4] = [
    // Default
    [
        k!(Escape), k!(AudioMute), k!(AudioVolDown), k!(AudioVolUp), k!(MediaPrevTrack), k!(MediaPlayPause), k!(MediaNextTrack), k!(BrightnessDown), k!(BrightnessUp), wm!(P, ModifierCombination::LGUI), k!(F17), k!(PrintScreen), a!(No), k!(Delete),
        k!(Grave), k!(Kc1), k!(Kc2), k!(Kc3), k!(Kc4), k!(Kc5), k!(Kc6), k!(Kc7), k!(Kc8), k!(Kc9), k!(Kc0), k!(Minus), k!(Equal), k!(Backspace),
        k!(Tab), k!(Q), k!(W), k!(E), k!(R), k!(T), k!(Y), k!(U), k!(I), k!(O), k!(P), k!(LeftBracket), k!(RightBracket), k!(Backslash),
        k!(CapsLock), k!(A), k!(S), k!(D), k!(F), k!(G), k!(H), k!(J), k!(K), k!(L), k!(Semicolon), k!(Quote), k!(Enter),
        k!(LShift), k!(Z), k!(X), k!(C), k!(V), k!(B), k!(N), k!(M), k!(Comma), k!(Dot), k!(Slash), k!(RShift),
        k!(LCtrl), mo!(1), k!(LGui), k!(LAlt), k!(Space), k!(RAlt), k!(RCtrl), k!(Left), k!(Up), k!(Right),
        k!(Down),
    ],
    // Fn
    [
        tg!(2), k!(F1), k!(F2), k!(F3), k!(F4), k!(F5), k!(F6), k!(F7), k!(F8), k!(F9), k!(F10), k!(F11), k!(F12), k!(Insert),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), k!(Pause), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), k!(ScrollLock), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), wm!(Pause, ModifierCombination::LCTRL), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), k!(F15), a!(Transparent), k!(F16), k!(Home), k!(PageUp), k!(End),
        k!(PageDown),
    ],
    // Fn lock
    [
        k!(Escape), k!(F1), k!(F2), k!(F3), k!(F4), k!(F5), k!(F6), k!(F7), k!(F8), k!(F9), k!(F10), k!(F11), k!(F12), k!(Delete),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), mo!(3), a!(Transparent), a!(Transparent), k!(Space), a!(Transparent), k!(RCtrl), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent),
    ],
    // Temporary Fn unlock
    [
        tg!(2), k!(AudioMute), k!(AudioVolDown), k!(AudioVolUp), k!(MediaPrevTrack), k!(MediaPlayPause), k!(MediaNextTrack), k!(BrightnessDown), k!(BrightnessUp), wm!(P, ModifierCombination::LGUI), k!(F17), k!(PrintScreen), a!(No), k!(Insert),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), k!(Pause), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), k!(ScrollLock), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), wm!(Pause, ModifierCombination::LCTRL), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), k!(F15), a!(Transparent), k!(F16), k!(Home), k!(PageUp), k!(End),
        k!(PageDown),
    ],
];

pub const fn default_keymap() -> [[[KeyAction; 18]; 8]; 4] {
    let mut map = [[[a!(No); 18]; 8]; 4];
    let mut layer = 0;
    while layer < LAYERS.len() {
        let mut key = 0;
        while key < POSITIONS.len() {
            let (row, col) = POSITIONS[key];
            map[layer][row][col] = LAYERS[layer][key];
            key += 1;
        }
        layer += 1;
    }
    map
}
