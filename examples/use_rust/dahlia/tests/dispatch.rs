use rmk::channel::USB_REPORT_CHANNEL;
use rmk::config::{BehaviorConfig, PositionalConfig};
use rmk::event::KeyboardEvent;
use rmk::hid::Report;
use rmk::keyboard::Keyboard;
use rmk::types::connection::UsbState;
use rmk::{KeymapData, initialize_keymap};

use crate::dispatch::dispatch;
use crate::keymap;

#[test]
fn every_transition_is_submitted_before_dispatch_returns() {
    rmk::test_support::test_block_on(async {
        let mut data = KeymapData::new(keymap::default_keymap());
        let mut behavior = BehaviorConfig::default();
        let positional = PositionalConfig::default();
        let keymap = initialize_keymap(&mut data, &mut behavior, &positional).await;
        let mut keyboard = Keyboard::new(&keymap);
        rmk::state::set_usb_state(UsbState::Configured);
        // A first: this is the physical key from the diagnostic snapshot.
        let keys = [
            (4, 1, 0x04),
            (6, 15, 0x39),
            (6, 15, 0x39),
            (5, 1, 0x1e),
            (6, 4, 0x1f),
            (6, 2, 0x20),
            (6, 3, 0x21),
            (3, 3, 0x22),
            (3, 8, 0x23),
            (6, 8, 0x24),
            (6, 5, 0x25),
            (6, 9, 0x26),
            (6, 6, 0x27),
            (3, 6, 0x2d),
            (0, 8, 0x2e),
        ];
        for (row, col, expected) in keys {
            for pressed in [true, false] {
                let mut submitted = 0;
                dispatch(&mut keyboard, KeyboardEvent::key(row, col, pressed), |report| {
                    let Report::KeyboardReport(report) = report else {
                        panic!("expected a keyboard report")
                    };
                    assert_eq!(report.keycodes[0], if pressed { expected } else { 0 });
                    submitted += 1;
                })
                .await;
                assert_eq!(submitted, 1, "each transition must submit immediately");
                assert!(USB_REPORT_CHANNEL.is_empty(), "no reports may wait for later strokes");
            }
        }
    });
}

#[test]
fn direct_dispatch_preserves_fn_layer_press_and_release() {
    rmk::test_support::test_block_on(async {
        let mut data = KeymapData::new(keymap::default_keymap());
        let mut behavior = BehaviorConfig::default();
        let positional = PositionalConfig::default();
        let keymap = initialize_keymap(&mut data, &mut behavior, &positional).await;
        let mut keyboard = Keyboard::new(&keymap);
        rmk::state::set_usb_state(UsbState::Configured);
        dispatch(&mut keyboard, KeyboardEvent::key(0, 16, true), |_| {}).await; // Fn
        let mut sent = 0;
        dispatch(&mut keyboard, KeyboardEvent::key(0, 2, true), |report| {
            let Report::KeyboardReport(report) = report else {
                panic!("expected F1")
            };
            assert_eq!(report.keycodes[0], 0x3a);
            sent += 1;
        })
        .await;
        assert_eq!(sent, 1);
        dispatch(&mut keyboard, KeyboardEvent::key(0, 2, false), |_| {}).await;
        dispatch(&mut keyboard, KeyboardEvent::key(0, 16, false), |_| {}).await;
        dispatch(&mut keyboard, KeyboardEvent::key(0, 2, true), |report| {
            let Report::MediaKeyboardReport(report) = report else {
                panic!("expected mute after releasing Fn")
            };
            let usage = report.usage_id;
            assert_eq!(usage, 0xe2);
        })
        .await;
        assert!(USB_REPORT_CHANNEL.is_empty());
    });
}

#[test]
fn shift_number_chords_preserve_modifiers_and_release() {
    rmk::test_support::test_block_on(async {
        let mut data = KeymapData::new(keymap::default_keymap());
        let mut behavior = BehaviorConfig::default();
        let positional = PositionalConfig::default();
        let keymap = initialize_keymap(&mut data, &mut behavior, &positional).await;
        let mut keyboard = Keyboard::new(&keymap);
        rmk::state::set_usb_state(UsbState::Configured);
        for (row, col, modifier) in [(5, 7, 0x02), (7, 7, 0x20)] {
            for (event, expected_mod, expected_key) in [
                (KeyboardEvent::key(row, col, true), modifier, 0),
                (KeyboardEvent::key(5, 1, true), modifier, 0x1e),
                (KeyboardEvent::key(5, 1, false), modifier, 0),
                (KeyboardEvent::key(row, col, false), 0, 0),
            ] {
                let mut sent = 0;
                dispatch(&mut keyboard, event, |report| {
                    let Report::KeyboardReport(report) = report else {
                        panic!("expected keyboard report")
                    };
                    assert_eq!(report.modifier, expected_mod);
                    assert_eq!(report.keycodes, [expected_key, 0, 0, 0, 0, 0]);
                    sent += 1;
                })
                .await;
                assert_eq!(sent, 1);
                assert!(USB_REPORT_CHANNEL.is_empty());
            }
        }
    });
}

#[test]
fn fn_lock_reporting_tracks_toggle_and_temporary_unlock() {
    rmk::test_support::test_block_on(async {
        let mut data = KeymapData::new(keymap::default_keymap());
        let mut behavior = BehaviorConfig::default();
        let positional = PositionalConfig::default();
        let keymap = initialize_keymap(&mut data, &mut behavior, &positional).await;
        let mut keyboard = Keyboard::new(&keymap);
        let mut protocol = crate::protocol::Protocol::new();
        rmk::state::set_usb_state(UsbState::Configured);
        for (row, col, pressed, locked) in [
            (0, 16, true, false), // Fn alone is momentary, not locked.
            (1, 1, true, false),  // RMK toggles on Escape release.
            (1, 1, false, true),
            (0, 16, false, true),
            (0, 16, true, true), // Fn temporarily unlocks, preserving lock state.
            (0, 16, false, true),
            (0, 16, true, true),
            (1, 1, true, true),   // Escape is held on the temporary-unlock layer.
            (1, 1, false, false), // Toggle off while temporary-unlock layer is still active.
            (0, 16, false, false),
        ] {
            dispatch(&mut keyboard, KeyboardEvent::key(row, col, pressed), |_| {}).await;
            protocol.fn_lock = keymap.is_layer_active(2);
            let mut request = [0u8; 36];
            request[..6].copy_from_slice(&[6, 0, 34, 0, 0x20, 0x13]);
            for bytes in [
                &[5, 0, 0x3f, 3, 0x20, 6, 0][..],
                &request[..],
                &[5, 0, 0x3f, 2, 0x20, 6, 0][..],
                &[6, 0][..],
            ] {
                protocol.start_write(0x15);
                for &byte in bytes {
                    protocol.receive(byte);
                }
                protocol.finish_write();
            }
            protocol.start_read(0x15);
            assert_eq!(&protocol.response[..9], &[17, 0, 0x20, 0x13, 0, 15, 0, 0, 0]);
            assert_eq!(
                &protocol.response[9..13],
                &(u32::from(locked) | 4).to_le_bytes(),
                "after ({row}, {col}), pressed={pressed}"
            );
            assert_eq!(&protocol.response[13..17], &[3, 0, 0, 0]);
            protocol.finish_read(17);
        }
    });
}

#[test]
fn restored_fn_lock_turns_off_with_one_chord() {
    rmk::test_support::test_block_on(async {
        let mut data = KeymapData::new(keymap::default_keymap());
        let mut behavior = BehaviorConfig::default();
        let positional = PositionalConfig::default();
        let keymap = initialize_keymap(&mut data, &mut behavior, &positional).await;
        keymap.set_layer_active(2, true);
        assert!(keymap.is_layer_active(2));
        let mut keyboard = Keyboard::new(&keymap);
        for (row, col, pressed) in [(0, 16, true), (1, 1, true), (1, 1, false), (0, 16, false)] {
            dispatch(&mut keyboard, KeyboardEvent::key(row, col, pressed), |_| {}).await;
        }
        assert!(!keymap.is_layer_active(2));
        assert!(!keymap.is_layer_active(3));
    });
}
