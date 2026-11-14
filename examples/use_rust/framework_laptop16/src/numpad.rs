// Copyright 2022–2023 Framework Computer
// SPDX-License-Identifier: GPL-2.0-or-later
// Matrix positions and default keymaps derived from Framework QMK firmware.
use rmk::{k, types::action::KeyAction};
pub const ROW: usize = 4;
pub const COL: usize = 8;
pub const PID: u16 = 0x0014;
#[rustfmt::skip]
pub const fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; 1] {
    [
        [
            [k!(NumLock), k!(Kp7), k!(Kp4), k!(Kp1), k!(KpSlash), k!(Kp8), k!(Kp5), k!(Kp2)],
            [k!(Kp0), k!(KpAsterisk), k!(Kp9), k!(Kp6), k!(Kp3), k!(KpDot), k!(KpMinus), k!(KpPlus)],
            [k!(KpEnter), k!(Escape), k!(Calculator), KeyAction::No, k!(Backspace), KeyAction::No, KeyAction::No, KeyAction::No],
            [KeyAction::No, KeyAction::No, KeyAction::No, KeyAction::No, k!(Equal), KeyAction::No, KeyAction::No, KeyAction::No],
        ],
    ]
}
