//! Dahlia HID-over-I2C framing and the EC's IT8801 register interface.

mod build_info {
    include!(concat!(env!("OUT_DIR"), "/build_info.rs"));
}

pub const FIRMWARE_VERSION: u16 = 0x0126;

pub const REPORT_DESCRIPTOR: &[u8] = &[
    // Keyboard, report 1: modifiers, reserved byte, six keycodes; LED output.
    0x05, 0x01, 0x09, 0x06, 0xa1, 0x01, 0x85, 0x01, 0x05, 0x07, 0x19, 0xe0, 0x29, 0xe7, 0x15, 0x00, 0x25, 0x01, 0x75,
    0x01, 0x95, 0x08, 0x81, 0x02, 0x75, 0x08, 0x95, 0x01, 0x81, 0x01, 0x19, 0x00, 0x29, 0xdd, 0x15, 0x00, 0x26, 0xdd,
    0x00, 0x75, 0x08, 0x95, 0x06, 0x81, 0x00, 0x05, 0x08, 0x19, 0x01, 0x29, 0x05, 0x15, 0x00, 0x25, 0x01, 0x75, 0x01,
    0x95, 0x05, 0x91, 0x02, 0x75, 0x03, 0x95, 0x01, 0x91, 0x01,
    0xc0, // Consumer control, report 2: one 16-bit usage.
    0x05, 0x0c, 0x09, 0x01, 0xa1, 0x01, 0x85, 0x02, 0x19, 0x00, 0x2a, 0xff, 0x03, 0x15, 0x00, 0x26, 0xff, 0x03, 0x75,
    0x10, 0x95, 0x01, 0x81, 0x00, 0xc0, // Vendor Feature report 0x20: 31 opaque command/response bytes.
    0x06, 0x00, 0xff, 0x09, 0x03, 0xa1, 0x01, 0x85, 0x20, 0x15, 0x00, 0x26, 0xff, 0x00, 0x75, 0x08, 0x95, 0x1f, 0xb2,
    0x02, 0x01, 0xc0,
];

pub const HID_DESCRIPTOR: [u8; 30] = [
    30,
    0,
    0,
    1,
    REPORT_DESCRIPTOR.len() as u8,
    0,
    2,
    0,
    3,
    0,
    11,
    0,
    4,
    0,
    36,
    0,
    5,
    0,
    6,
    0,
    0xac,
    0x32,
    0x38,
    0,
    FIRMWARE_VERSION as u8,
    (FIRMWARE_VERSION >> 8) as u8,
    // Engineering firmware: ask BIOS to leave this image installed.
    1,
    0,
    0,
    0,
];

#[derive(Clone, Copy)]
pub struct Packet {
    pub bytes: [u8; 11],
    pub len: usize,
}

impl Packet {
    pub const fn new(id: u8, payload: &[u8]) -> Self {
        let mut bytes = [0; 11];
        bytes[0] = (payload.len() + 3) as u8;
        bytes[2] = id;
        let mut i = 0;
        while i < payload.len() {
            bytes[i + 3] = payload[i];
            i += 1;
        }
        Self {
            bytes,
            len: payload.len() + 3,
        }
    }
}

const EMPTY: Packet = Packet { bytes: [0; 11], len: 2 };
const KEY_UP: Packet = Packet::new(1, &[0; 8]);
const MEDIA_UP: Packet = Packet::new(2, &[0; 2]);
const QUEUE_SIZE: usize = 2;

pub struct Protocol {
    pub sovr: [u8; 3],
    pub leds: u8,
    pub fn_lock: bool,
    pub fn_ctrl_swap: bool,
    queue: [Packet; QUEUE_SIZE],
    head: usize,
    count: usize,
    keyboard: Packet,
    media: Packet,
    data: Packet,
    vendor: [u8; 34],
    vendor_selected: bool,
    pub hardware_info: [u8; 18],
    reset: bool,
    register: u16,
    io_register: u8,
    write: [u8; 40],
    written: usize,
    address: u8,
    reading_input: bool,
    read_address: u8,
    pub response: [u8; 128],
    pub response_len: usize,
    // Scans, matrix transitions, keyboard reports, input reads, consumed inputs.
    trace_counts: [u32; 5],
    trace_event: [u8; 3],
    trace_raw: [u32; 8],
    trace_wire: [u8; 8],
    trace_transferred: u32,
}

impl Default for Protocol {
    fn default() -> Self {
        Self::new()
    }
}

impl Protocol {
    pub const fn new() -> Self {
        Self {
            sovr: [255; 3],
            leds: 0,
            fn_lock: false,
            fn_ctrl_swap: false,
            queue: [EMPTY; QUEUE_SIZE],
            head: 0,
            count: 0,
            keyboard: KEY_UP,
            media: MEDIA_UP,
            data: EMPTY,
            vendor: [0; 34],
            vendor_selected: false,
            hardware_info: [0; 18],
            reset: false,
            register: 3,
            io_register: 0,
            write: [0; 40],
            written: 0,
            address: 0,
            reading_input: false,
            read_address: 0,
            response: [0; 128],
            response_len: 0,
            trace_counts: [0; 5],
            trace_event: [0; 3],
            trace_raw: [0; 8],
            trace_wire: [0; 8],
            trace_transferred: 0,
        }
    }

    pub fn note_scan(&mut self, raw: [u32; 8]) {
        self.trace_counts[0] = self.trace_counts[0].wrapping_add(1);
        self.trace_raw = raw;
    }

    pub fn note_event(&mut self, row: u8, col: u8, pressed: bool) {
        self.trace_counts[1] = self.trace_counts[1].wrapping_add(1);
        self.trace_event = [row, col, u8::from(pressed)];
    }

    pub fn pending(&self) -> bool {
        self.reset || self.count != 0
    }
    pub fn scanning(&self) -> bool {
        self.sovr[2] & 4 != 0
    }
    pub fn caps_lock_led(&self) -> bool {
        self.leds & 2 != 0
    }
    pub fn touchpad(&self) -> bool {
        self.sovr[2] & 1 != 0
    }

    pub fn enqueue(&mut self, packet: Packet) {
        match packet.bytes[2] {
            1 => {
                self.keyboard = packet;
                self.trace_counts[2] = self.trace_counts[2].wrapping_add(1);
            }
            2 => self.media = packet,
            _ => return,
        }
        if self.reset {
            return;
        }
        // Keep one current state per report ID, rather than replaying old
        // keystrokes when the EC drains reports more slowly than we scan.
        for offset in 0..self.count {
            let index = (self.head + offset) % QUEUE_SIZE;
            if self.queue[index].bytes[2] == packet.bytes[2] {
                self.queue[index] = packet;
                return;
            }
        }
        self.queue[(self.head + self.count) % QUEUE_SIZE] = packet;
        self.count += 1;
    }

    pub fn start_write(&mut self, address: u8) {
        self.address = address;
        self.written = 0;
    }

    pub fn receive(&mut self, byte: u8) {
        if self.written < self.write.len() {
            self.write[self.written] = byte;
            self.written += 1;
        }
    }

    // Called at STOP or repeated START, before preparing the next read.
    pub fn finish_write(&mut self) {
        let n = self.written;
        self.written = 0;
        if self.address == 0x38 {
            if n == 0 {
                return;
            }
            self.io_register = self.write[0];
            for i in 1..n {
                if (5..=7).contains(&self.io_register) {
                    self.sovr[(self.io_register - 5) as usize] = self.write[i];
                }
                self.io_register = self.io_register.wrapping_add(1);
            }
            return;
        }
        if n < 2 {
            return;
        }
        self.register = u16::from_le_bytes([self.write[0], self.write[1]]);
        match self.register {
            5 if n >= 4 => match self.write[3] {
                1 => {
                    self.head = 0;
                    self.count = 0;
                    self.reset = true;
                    self.register = 3;
                }
                2 => {
                    self.vendor_selected = n >= 7 && self.write[2] == 0x3f && self.write[4] == 0x20;
                    let packet = match self.write[2] & 15 {
                        1 => self.keyboard,
                        2 => self.media,
                        _ => EMPTY,
                    };
                    self.data = packet;
                    self.register = 6;
                }
                // The SET_REPORT payload arrives in a separate data-register write.
                _ => self.register = 3,
            },
            4 | 6 if n > 2 => {
                if self.register == 6 && n >= 6 && self.write[4] == 0x20 {
                    let length = u16::from_le_bytes([self.write[2], self.write[3]]) as usize;
                    if (4..=34).contains(&length) && length + 2 == n {
                        self.vendor = [0; 34];
                        let opcode = self.write[5];
                        self.vendor[2] = 0x20;
                        self.vendor[3] = opcode;
                        self.vendor[0] = match opcode {
                            1 => {
                                self.vendor[5..7].copy_from_slice(&FIRMWARE_VERSION.to_le_bytes());
                                self.vendor[7] = build_info::GIT_DIRTY;
                                self.vendor[8..16].copy_from_slice(&build_info::GIT_HASH);
                                16
                            }
                            2 => {
                                self.vendor[5..23].copy_from_slice(&self.hardware_info);
                                23
                            }
                            0x10 => {
                                if length < 5 {
                                    self.vendor[4] = 1;
                                } else {
                                    self.fn_ctrl_swap = self.write[6] != 0;
                                }
                                5
                            }
                            0x11 => {
                                self.vendor[5] = u8::from(self.fn_ctrl_swap);
                                6
                            }
                            0x13 => {
                                // Fn Lock is supported for reporting only.
                                self.vendor[5..9].copy_from_slice(&1u32.to_le_bytes());
                                self.vendor[9..13].copy_from_slice(&u32::from(self.fn_lock).to_le_bytes());
                                // Writable mask remains zero.
                                17
                            }
                            _ => {
                                self.vendor[4] = 0xff;
                                5
                            }
                        };
                    }
                } else if n >= 6 && self.write[4] == 1 {
                    self.leds = self.write[5];
                }
                // Output payloads do not select a register for the next bare
                // input read. The EC sends LED writes between key reports.
                self.register = 3;
            }
            _ => {}
        }
    }

    pub fn start_read(&mut self, address: u8) {
        self.reading_input = false;
        self.read_address = address;
        if address == 0x38 {
            self.response_len = self.response.len();
            let mut register = self.io_register;
            for byte in &mut self.response {
                *byte = match register {
                    5..=7 => self.sovr[(register - 5) as usize],
                    0xfe => 0x83,
                    0xff => 0x12,
                    _ => 0,
                };
                register = register.wrapping_add(1);
            }
            return;
        }
        // Engineering diagnostic register; accessed atomically with a combined
        // write/read so the EC's normal input reads cannot interleave.
        if self.register == 7 {
            self.response[..4].copy_from_slice(b"RMK4");
            for (i, count) in self.trace_counts.iter().enumerate() {
                self.response[4 + i * 4..8 + i * 4].copy_from_slice(&count.to_le_bytes());
            }
            self.response[24..27].copy_from_slice(&self.trace_event);
            self.response[27] = self.count as u8;
            self.response[28..32].copy_from_slice(&self.trace_transferred.to_le_bytes());
            self.response[32..40].copy_from_slice(&self.keyboard.bytes[3..11]);
            self.response[40..48].copy_from_slice(&self.trace_wire);
            for (i, row) in self.trace_raw.iter().enumerate() {
                self.response[48 + i * 4..52 + i * 4].copy_from_slice(&row.to_le_bytes());
            }
            self.response_len = 80;
            return;
        }
        let bytes: &[u8] = match self.register {
            1 => &HID_DESCRIPTOR,
            2 => REPORT_DESCRIPTOR,
            3 => {
                self.reading_input = true;
                self.trace_counts[3] = self.trace_counts[3].wrapping_add(1);
                if self.reset {
                    &EMPTY.bytes[..2]
                } else if self.count != 0 {
                    let packet = &self.queue[self.head];
                    if packet.bytes[2] == 1 {
                        self.trace_wire.copy_from_slice(&packet.bytes[3..11]);
                    }
                    &packet.bytes[..packet.len]
                } else {
                    &[2, 0]
                }
            }
            6 if self.vendor_selected => &self.vendor[..self.vendor[0] as usize],
            6 => &self.data.bytes[..self.data.len],
            _ => &[0, 0],
        };
        self.response_len = bytes.len();
        self.response[..bytes.len()].copy_from_slice(bytes);
    }

    pub fn finish_read(&mut self, transferred: usize) -> bool {
        if self.read_address == 0x38 {
            self.io_register = self.io_register.wrapping_add(transferred as u8);
            return false;
        }
        // Bare host reads always select the input register.
        self.register = 3;
        if !self.reading_input {
            return false;
        }
        self.reading_input = false;
        self.trace_transferred = transferred as u32;
        if transferred < self.response_len {
            // Pulse IRQ again so a short read does not strand pending input.
            return transferred != 0;
        }
        if self.reset {
            self.reset = false;
            self.enqueue(self.keyboard);
            self.enqueue(self.media);
        } else if self.count != 0
            && self.queue[self.head].bytes[..self.response_len] == self.response[..self.response_len]
        {
            // A state changed during this read must remain pending.
            self.head = (self.head + 1) % QUEUE_SIZE;
            self.count -= 1;
            self.trace_counts[4] = self.trace_counts[4].wrapping_add(1);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn write(p: &mut Protocol, addr: u8, data: &[u8]) {
        p.start_write(addr);
        for &b in data {
            p.receive(b);
        }
        p.finish_write();
    }
    #[test]
    fn descriptors_match_wire_lengths_and_board_identity() {
        assert_eq!(HID_DESCRIPTOR[4] as usize, REPORT_DESCRIPTOR.len());
        assert_eq!(&HID_DESCRIPTOR[20..26], &[0xac, 0x32, 0x38, 0, 0x26, 1]);
        let mut p = Protocol::new();
        write(&mut p, 0x15, &[1, 0]);
        p.start_read(0x15);
        assert_eq!(&p.response[..p.response_len], HID_DESCRIPTOR);
    }
    #[test]
    fn short_reads_retry_without_losing_latest_state() {
        let mut p = Protocol::new();
        p.enqueue(Packet::new(1, &[0, 0, 4, 0, 0, 0, 0, 0]));
        p.start_read(0x15);
        assert!(p.finish_read(2));
        assert!(p.pending());
        p.enqueue(KEY_UP);
        p.start_read(0x15);
        assert_eq!(p.response[5], 0);
        p.finish_read(11);
        assert!(!p.pending());
    }
    #[test]
    fn release_arriving_during_read_is_delivered_next() {
        let mut p = Protocol::new();
        p.enqueue(Packet::new(1, &[0, 0, 4, 0, 0, 0, 0, 0]));
        p.start_read(0x15);
        p.enqueue(KEY_UP);
        p.finish_read(11);
        assert!(p.pending());
        p.start_read(0x15);
        assert_eq!(p.response[5], 0);
        p.finish_read(11);
        assert!(!p.pending());
    }
    #[test]
    fn reset_marker_precedes_resynchronized_state() {
        let mut p = Protocol::new();
        p.enqueue(Packet::new(1, &[2, 0, 4, 0, 0, 0, 0, 0]));
        write(&mut p, 0x15, &[5, 0, 0, 1]);
        p.start_read(0x15);
        assert_eq!(&p.response[..p.response_len], &[0, 0]);
        p.finish_read(2);
        p.start_read(0x15);
        assert_eq!(&p.response[2..6], &[1, 2, 0, 4]);
    }
    #[test]
    fn ioexpander_identity_and_controls() {
        let mut p = Protocol::new();
        write(&mut p, 0x38, &[0xfe]);
        p.start_read(0x38);
        assert_eq!(&p.response[..2], &[0x83, 0x12]);
        write(&mut p, 0x38, &[7, 0]);
        assert!(!p.scanning());
        assert!(!p.touchpad());
        write(&mut p, 0x38, &[7, 5]);
        assert!(p.scanning());
        assert!(p.touchpad());
    }
    #[test]
    fn get_report_returns_current_keyboard_state() {
        let mut p = Protocol::new();
        p.enqueue(Packet::new(1, &[1, 0, 5, 0, 0, 0, 0, 0]));
        write(&mut p, 0x15, &[5, 0, 0x11, 2, 6, 0]);
        p.start_read(0x15);
        assert_eq!(&p.response[..6], &[11, 0, 1, 1, 0, 5]);
    }
    #[test]
    fn reset_coalesces_events_until_marker_is_read() {
        let mut p = Protocol::new();
        write(&mut p, 0x15, &[5, 0, 0, 1]);
        for _ in 0..32 {
            p.enqueue(Packet::new(1, &[1, 0, 4, 0, 0, 0, 0, 0]));
            p.enqueue(KEY_UP);
        }
        p.start_read(0x15);
        p.finish_read(2);
        p.start_read(0x15);
        assert_eq!(&p.response[..11], &KEY_UP.bytes);
        p.finish_read(11);
        p.start_read(0x15);
        assert_eq!(&p.response[..5], &MEDIA_UP.bytes[..5]);
        p.finish_read(5);
        assert!(!p.pending());
    }
    #[test]
    fn unread_typing_coalesces_without_blocking_or_losing_releases() {
        let mut p = Protocol::new();
        for _ in 0..100 {
            p.enqueue(Packet::new(1, &[0, 0, 4, 0, 0, 0, 0, 0]));
            p.enqueue(KEY_UP);
            p.enqueue(Packet::new(2, &[0xe9, 0]));
            p.enqueue(MEDIA_UP);
        }
        assert_eq!(p.count, 2);
        p.start_read(0x15);
        assert_eq!(&p.response[..11], &KEY_UP.bytes);
        p.finish_read(11);
        p.start_read(0x15);
        assert_eq!(&p.response[..5], &MEDIA_UP.bytes[..5]);
        p.finish_read(5);
        assert!(!p.pending());
    }
    #[test]
    fn io_reads_advance_only_by_bytes_transferred() {
        let mut p = Protocol::new();
        write(&mut p, 0x38, &[0xfe]);
        p.start_read(0x38);
        p.finish_read(1);
        p.start_read(0x38);
        assert_eq!(p.response[0], 0x12);
    }
    #[test]
    fn unrelated_reads_do_not_destroy_get_report_response() {
        let mut p = Protocol::new();
        p.enqueue(Packet::new(2, &[0xe9, 0]));
        write(&mut p, 0x15, &[5, 0, 0x12, 2, 6, 0]);
        write(&mut p, 0x38, &[0xfe]);
        p.start_read(0x38);
        p.finish_read(2);
        write(&mut p, 0x15, &[6, 0]);
        p.start_read(0x15);
        assert_eq!(&p.response[..5], &[5, 0, 2, 0xe9, 0]);
    }
    #[test]
    fn bare_reads_return_input_after_descriptor_transaction() {
        let mut p = Protocol::new();
        p.enqueue(KEY_UP);
        write(&mut p, 0x15, &[1, 0]);
        p.start_read(0x15);
        p.finish_read(30);
        p.start_read(0x15);
        assert_eq!(&p.response[..11], &KEY_UP.bytes);
    }
}

#[cfg(test)]
mod slow_host_regression {
    use super::*;

    #[test]
    fn slow_host_reads_current_key_instead_of_previous_typing() {
        let mut p = Protocol::new();
        for key in [0x1a, 0x08, 0x15, 0x17, 0x1c] {
            p.enqueue(Packet::new(1, &[0, 0, key, 0, 0, 0, 0, 0]));
            p.enqueue(KEY_UP);
        }
        p.enqueue(Packet::new(1, &[0, 0, 0x1e, 0, 0, 0, 0, 0]));
        p.start_read(0x15);
        assert_eq!(p.response[5], 0x1e, "host must receive 1, not an old W");
    }
}

#[cfg(test)]
mod diagnostic_tests {
    use super::*;

    #[test]
    fn diagnostic_snapshot_identifies_each_stage_without_consuming_input() {
        let mut p = Protocol::new();
        let mut raw = [0; 8];
        raw[4] = 2; // A on the physical matrix.
        p.note_scan(raw);
        p.note_event(4, 1, true);
        p.enqueue(Packet::new(1, &[0, 0, 4, 0, 0, 0, 0, 0]));
        p.start_read(0x15);
        p.finish_read(11);
        p.start_write(0x15);
        p.receive(7);
        p.receive(0);
        p.finish_write();
        p.start_read(0x15);
        assert_eq!(&p.response[..4], b"RMK4");
        for i in 0..5 {
            assert_eq!(
                u32::from_le_bytes(p.response[4 + i * 4..8 + i * 4].try_into().unwrap()),
                1
            );
        }
        assert_eq!(&p.response[24..27], &[4, 1, 1]);
        assert_eq!(p.response[34], 4);
        assert_eq!(p.response[42], 4);
        assert_eq!(p.response[64], 2);
        assert!(!p.finish_read(80));
        assert!(!p.pending());
    }
}

#[cfg(test)]
mod caps_lock_tests {
    use super::*;

    #[test]
    fn ec_led_write_does_not_redirect_the_next_key_read() {
        let mut p = Protocol::new();
        for leds in [2, 0] {
            p.start_write(0x15);
            // Exact EC wOutputRegister write: register, length, ID, LED bits.
            for b in [4, 0, 4, 0, 1, leds] {
                p.receive(b);
            }
            p.finish_write();
            assert_eq!(p.leds, leds);
            p.enqueue(KEY_UP);
            p.start_read(0x15);
            assert_eq!(&p.response[..p.response_len], &KEY_UP.bytes[..11]);
            assert!(p.finish_read(11));
            assert!(!p.pending());
        }
    }

    #[test]
    fn led_data_register_write_preserves_input_and_clears_caps() {
        let mut p = Protocol::new();
        for leds in [2, 1, 0] {
            p.start_write(0x15);
            for b in [6, 0, 4, 0, 1, leds] {
                p.receive(b);
            }
            p.finish_write();
            assert_eq!(p.caps_lock_led(), leds == 2);
            p.enqueue(KEY_UP);
            p.start_read(0x15);
            assert_eq!(&p.response[..p.response_len], &KEY_UP.bytes[..11]);
            p.finish_read(11);
            assert!(!p.pending());
        }
    }
}

#[cfg(test)]
mod vendor_tests {
    use super::*;

    fn write(p: &mut Protocol, bytes: &[u8]) {
        p.start_write(0x15);
        for &byte in bytes {
            p.receive(byte);
        }
        p.finish_write();
    }

    fn request(p: &mut Protocol, opcode: u8) {
        write(p, &[5, 0, 0x3f, 3, 0x20, 6, 0]);
        let mut bytes = [0; 36];
        bytes[..6].copy_from_slice(&[6, 0, 34, 0, 0x20, opcode]);
        write(p, &bytes);
    }

    fn read_reply(p: &mut Protocol) {
        write(p, &[5, 0, 0x3f, 2, 0x20, 6, 0]);
        write(p, &[6, 0]);
        p.start_read(0x15);
    }

    #[test]
    fn swap_commands_round_trip_and_reject_missing_value() {
        let mut p = Protocol::new();
        write(&mut p, &[6, 0, 5, 0, 0x20, 0x10, 1]);
        assert!(p.fn_ctrl_swap);
        request(&mut p, 0x11);
        read_reply(&mut p);
        assert_eq!(&p.response[..6], &[6, 0, 0x20, 0x11, 0, 1]);
        write(&mut p, &[6, 0, 4, 0, 0x20, 0x10]);
        read_reply(&mut p);
        assert_eq!(p.response[4], 1);
        assert!(p.fn_ctrl_swap);
        write(&mut p, &[6, 0, 5, 0, 0x20, 0x10, 0]);
        assert!(!p.fn_ctrl_swap);
        assert!(!Protocol::new().fn_ctrl_swap);
    }

    #[test]
    fn framework_tool_firmware_query_returns_version_and_commit() {
        let mut p = Protocol::new();
        request(&mut p, 1);
        read_reply(&mut p);
        assert_eq!(p.response_len, 16);
        assert_eq!(&p.response[..5], &[16, 0, 0x20, 1, 0]);
        assert_eq!(&p.response[5..7], &HID_DESCRIPTOR[24..26]);
        assert!(p.response[7] <= 1);
        assert_ne!(&p.response[8..16], &[0; 8]);
    }

    #[test]
    fn hardware_query_returns_chip_memory_and_unique_id() {
        let mut p = Protocol::new();
        p.hardware_info = [0x89, 0x04, 64, 0, 40, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        request(&mut p, 2);
        read_reply(&mut p);
        assert_eq!(p.response_len, 23);
        assert_eq!(&p.response[..5], &[23, 0, 0x20, 2, 0]);
        assert_eq!(&p.response[5..23], &p.hardware_info);
    }

    #[test]
    fn key_and_led_transactions_between_vendor_set_and_get_preserve_reply() {
        let mut p = Protocol::new();
        request(&mut p, 1);
        p.enqueue(Packet::new(1, &[2, 0, 0x1e, 0, 0, 0, 0, 0]));
        p.start_read(0x15);
        assert_eq!(&p.response[..6], &[11, 0, 1, 2, 0, 0x1e]);
        p.finish_read(11);
        write(&mut p, &[4, 0, 4, 0, 1, 0]);
        read_reply(&mut p);
        assert_eq!(&p.response[..5], &[16, 0, 0x20, 1, 0]);
        p.finish_read(16);
        p.enqueue(KEY_UP);
        p.start_read(0x15);
        assert_eq!(&p.response[..11], &KEY_UP.bytes);
    }

    #[test]
    fn malformed_vendor_lengths_do_not_execute_or_redirect_input() {
        for length in [0, 3, 33, 35, 0xffff] {
            let mut p = Protocol::new();
            request(&mut p, 1);
            let mut bytes = [0; 36];
            bytes[..6].copy_from_slice(&[6, 0, length as u8, (length >> 8) as u8, 0x20, 2]);
            write(&mut p, &bytes);
            p.enqueue(KEY_UP);
            p.start_read(0x15);
            assert_eq!(&p.response[..11], &KEY_UP.bytes);
            p.finish_read(11);
            read_reply(&mut p);
            assert_eq!(&p.response[..5], &[16, 0, 0x20, 1, 0]);
        }
    }

    #[test]
    fn flags_query_reports_fn_lock_as_read_only_and_off_at_boot() {
        let mut p = Protocol::new();
        request(&mut p, 0x13);
        read_reply(&mut p);
        assert_eq!(p.response_len, 17);
        assert_eq!(
            &p.response[..17],
            &[17, 0, 0x20, 0x13, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn unknown_vendor_command_is_reported_without_changing_input_or_leds() {
        let mut p = Protocol::new();
        p.enqueue(Packet::new(1, &[2, 0, 0x1e, 0, 0, 0, 0, 0]));
        write(&mut p, &[4, 0, 4, 0, 1, 2]);
        request(&mut p, 0x12);
        read_reply(&mut p);
        assert_eq!(&p.response[..5], &[5, 0, 0x20, 0x12, 0xff]);
        assert!(!p.finish_read(17));
        assert!(p.pending());
        assert!(p.caps_lock_led());
        p.start_read(0x15);
        assert_eq!(&p.response[..6], &[11, 0, 1, 2, 0, 0x1e]);
    }
}
