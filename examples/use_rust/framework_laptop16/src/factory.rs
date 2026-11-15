// SPDX-License-Identifier: GPL-2.0-or-later
use embassy_time::Timer;
use embassy_usb::class::hid::device::{Config, HidReaderWriter, State};
use embassy_usb::class::hid::{HidBootProtocol, HidSubclass};
use embassy_usb::driver::Driver;
use rmk::core_traits::Runnable;
use static_cell::StaticCell;

// Match QMK's raw HID usage page, usage ID, and 32-byte input/output reports.
const REPORT_DESCRIPTOR: &[u8] = &[
    0x06, 0x60, 0xff, 0x09, 0x61, 0xa1, 0x01, 0x09, 0x62, 0x15, 0x00, 0x26, 0xff, 0x00, 0x95, 0x20,
    0x75, 0x08, 0x81, 0x02, 0x09, 0x63, 0x15, 0x00, 0x26, 0xff, 0x00, 0x95, 0x20, 0x75, 0x08, 0x91,
    0x02, 0xc0,
];

pub struct FactoryHid<D: Driver<'static>> {
    hid: HidReaderWriter<'static, D, 32, 32>,
}

impl<D: Driver<'static>> FactoryHid<D> {
    pub fn new(builder: &mut embassy_usb::Builder<'static, D>) -> Self {
        static STATE: StaticCell<State<'static>> = StaticCell::new();
        Self {
            hid: HidReaderWriter::new(
                builder,
                STATE.init(State::new()),
                Config {
                    report_descriptor: REPORT_DESCRIPTOR,
                    request_handler: None,
                    poll_ms: 1,
                    max_packet_size: 32,
                    hid_subclass: HidSubclass::No,
                    hid_boot_protocol: HidBootProtocol::None,
                },
            ),
        }
    }
}

impl<D: Driver<'static>> Runnable for FactoryHid<D> {
    async fn run(&mut self) -> ! {
        loop {
            let mut report = [0u8; 32];
            match self.hid.read(&mut report).await {
                // qmk_hid includes the report-ID placeholder in its 32-byte write.
                Ok(31 | 32) if report[..2] == [0x0b, 0xfe] => rmk::boot::jump_to_bootloader(),
                Ok(_) => {}
                Err(_) => {
                    // A disabled endpoint can fail immediately while USB is disconnected.
                    Timer::after_millis(10).await;
                }
            }
        }
    }
}
