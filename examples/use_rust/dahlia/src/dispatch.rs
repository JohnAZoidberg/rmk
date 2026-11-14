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
