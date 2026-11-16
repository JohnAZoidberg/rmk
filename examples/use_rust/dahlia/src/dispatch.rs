//! Process one matrix transition and submit all reports before the next scan.

use rmk::channel::USB_REPORT_CHANNEL;
use rmk::event::KeyboardEvent;
use rmk::hid::Report;
use rmk::keyboard::Keyboard;

pub async fn dispatch(keyboard: &mut Keyboard<'_>, event: KeyboardEvent, mut submit: impl FnMut(Report)) {
    keyboard.process_inner(event).await;
    while let Ok(report) = USB_REPORT_CHANNEL.try_receive() {
        submit(report);
    }
}

/// Keep each release paired with the binding selected on its press.
#[derive(Default)]
pub struct FnCtrlSwap {
    pressed_swapped: [bool; 2],
}

impl FnCtrlSwap {
    pub fn position(&mut self, row: usize, col: usize, pressed: bool, swapped: bool) -> (usize, usize) {
        let index = match (row, col) {
            (1, 14) => 0,
            (0, 16) => 1,
            _ => return (row, col),
        };
        if pressed {
            self.pressed_swapped[index] = swapped;
        }
        if self.pressed_swapped[index] {
            [(0, 16), (1, 14)][index]
        } else {
            (row, col)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FnCtrlSwap;

    #[test]
    fn swap_changes_both_keys_and_preserves_held_bindings() {
        let mut swap = FnCtrlSwap::default();
        assert_eq!(swap.position(1, 14, true, true), (0, 16));
        assert_eq!(swap.position(1, 14, false, false), (0, 16));
        assert_eq!(swap.position(0, 16, true, true), (1, 14));
        assert_eq!(swap.position(0, 16, false, false), (1, 14));
        assert_eq!(swap.position(0, 16, true, false), (0, 16));
        assert_eq!(swap.position(4, 1, true, true), (4, 1));
    }
}
