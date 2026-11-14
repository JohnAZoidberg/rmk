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
