use core::cell::RefCell;

use critical_section::Mutex;
use embassy_stm32::gpio::{Level, Output, OutputOpenDrain};
use embassy_stm32::interrupt::{self, InterruptExt};
use embassy_stm32::pac;
use embassy_time::{Duration, Instant, Timer};
use rmk::hid::Report;

use crate::protocol::{Packet, Protocol};

struct Bus {
    protocol: Protocol,
    irq: OutputOpenDrain<'static>,
    touchpad: Output<'static>,
    caps_led: Output<'static>,
    writing: bool,
    reading: bool,
    sent: usize,
    rearm_at: Instant,
}

static BUS: Mutex<RefCell<Option<Bus>>> = Mutex::new(RefCell::new(None));

pub fn init(irq: OutputOpenDrain<'static>, touchpad: Output<'static>, caps_led: Output<'static>) {
    let mut protocol = Protocol::new();
    // DBGMCU's ID register is only accessible while its clock is enabled.
    let debug_enabled = pac::RCC.dbgcfgr().read().dbgen();
    pac::RCC.dbgcfgr().modify(|w| w.set_dbgen(true));
    let _ = pac::RCC.dbgcfgr().read();
    let chip_id = pac::DBGMCU.idcode().read().dev_id();
    pac::RCC.dbgcfgr().modify(|w| w.set_dbgen(debug_enabled));
    // STM32U073 factory FLASHSIZE register, in KiB.
    let flash_kb = unsafe { core::ptr::read_volatile(0x1fff_6ea0 as *const u16) };
    protocol.hardware_info[..2].copy_from_slice(&chip_id.to_le_bytes());
    protocol.hardware_info[2..4].copy_from_slice(&flash_kb.to_le_bytes());
    protocol.hardware_info[4..6].copy_from_slice(&40u16.to_le_bytes());
    protocol.hardware_info[6..].copy_from_slice(&embassy_stm32::uid::uid());
    critical_section::with(|cs| {
        *BUS.borrow(cs).borrow_mut() = Some(Bus {
            protocol,
            irq,
            touchpad,
            caps_led,
            writing: false,
            reading: false,
            sent: 0,
            rearm_at: Instant::now(),
        });
    });
    let regs = pac::I2C1;
    regs.cr1().write(|w| w.set_pe(false));
    // HSI16/PCLK1 at 16 MHz, matching the working ZMK Fm+ timing.
    regs.timingr().write_value(pac::i2c::regs::Timingr(0x0010_040a));
    regs.oar1().write(|w| {
        w.set_oa1(0x15 << 1);
        w.set_oa1en(true);
    });
    regs.oar2().write(|w| {
        w.set_oa2(0x38);
        w.set_oa2en(true);
    });
    pac::RCC.apbenr2().modify(|w| w.set_syscfgen(true));
    pac::SYSCFG.cfgr1().modify(|w| {
        w.set_i2c_pb6_fmp(true);
        w.set_i2c_pb7_fmp(true);
    });
    // CR1.FMP (bit 24) is absent from the PAC's shared I2C-v2 register model.
    regs.cr1().write_value(pac::i2c::regs::Cr1(1 << 24));
    regs.cr1().modify(|w| {
        w.set_pe(true);
        w.set_rxie(true);
        w.set_addrie(true);
        w.set_nackie(true);
        w.set_stopie(true);
        w.set_errie(true);
    });
    interrupt::I2C1.set_priority(interrupt::Priority::P0);
    unsafe {
        interrupt::I2C1.enable();
    }
}

pub fn fn_ctrl_swap() -> bool {
    critical_section::with(|cs| BUS.borrow(cs).borrow().as_ref().unwrap().protocol.fn_ctrl_swap)
}

pub fn scanning() -> bool {
    critical_section::with(|cs| BUS.borrow(cs).borrow().as_ref().unwrap().protocol.scanning())
}

pub fn note_scan(raw: [u32; 8]) {
    critical_section::with(|cs| {
        BUS.borrow(cs).borrow_mut().as_mut().unwrap().protocol.note_scan(raw);
    });
}

pub fn note_event(row: u8, col: u8, pressed: bool) {
    critical_section::with(|cs| {
        BUS.borrow(cs)
            .borrow_mut()
            .as_mut()
            .unwrap()
            .protocol
            .note_event(row, col, pressed);
    });
}

impl Bus {
    fn finish_write(&mut self) {
        if self.writing {
            self.protocol.finish_write();
            self.touchpad.set_level(if self.protocol.touchpad() {
                Level::High
            } else {
                Level::Low
            });
            self.caps_led.set_level(if self.protocol.caps_lock_led() {
                Level::High
            } else {
                Level::Low
            });
            self.writing = false;
        }
    }

    fn finish_read(&mut self) {
        if self.reading {
            // TXE describes TXDR, not a count of bytes read by the EC. In
            // particular, do not subtract a byte from a completed input read.
            if self.protocol.finish_read(self.sent) {
                self.irq.set_high();
                self.rearm_at = Instant::now() + Duration::from_micros(20);
            }
            self.reading = false;
        }
    }
}

#[embassy_stm32::interrupt]
fn I2C1() {
    critical_section::with(|cs| {
        let mut guard = BUS.borrow(cs).borrow_mut();
        let Some(bus) = guard.as_mut() else {
            return;
        };
        let regs = pac::I2C1;
        let status = regs.isr().read();
        if status.rxne() {
            bus.protocol.receive(regs.rxdr().read().rxdata());
        }
        if status.stopf() || status.addr() {
            bus.finish_read();
            bus.finish_write();
        }
        if status.nackf() || status.stopf() || status.addr() {
            regs.cr1().modify(|w| w.set_txie(false));
            // Flush any unconsumed prefetch before the next transaction.
            regs.isr().write(|w| w.set_txe(true));
        }
        regs.icr().write(|w| {
            w.set_nackcf(status.nackf());
            w.set_stopcf(status.stopf());
            w.set_berrcf(status.berr());
            w.set_arlocf(status.arlo());
            w.set_ovrcf(status.ovr());
        });
        if status.addr() {
            let address = status.addcode();
            bus.sent = 0;
            if status.dir() == pac::i2c::vals::Dir::READ {
                bus.protocol.start_read(address);
                bus.reading = true;
                regs.cr1().modify(|w| w.set_txie(true));
            } else {
                bus.protocol.start_write(address);
                bus.writing = true;
            }
            regs.icr().write(|w| w.set_addrcf(true));
        }
        if regs.isr().read().txis() && bus.reading && !status.nackf() && !status.stopf() {
            let byte = if bus.sent < bus.protocol.response_len {
                bus.protocol.response[bus.sent]
            } else {
                0
            };
            regs.txdr().write(|w| w.set_txdata(byte));
            bus.sent += 1;
        }
    });
}

pub fn submit_report(report: Report) {
    let packet = match report {
        Report::KeyboardReport(r) => {
            let mut bytes = [0; 8];
            bytes[0] = r.modifier;
            bytes[2..].copy_from_slice(&r.keycodes);
            Packet::new(1, &bytes)
        }
        Report::MediaKeyboardReport(r) => Packet::new(2, &r.usage_id.to_le_bytes()),
        _ => return,
    };
    critical_section::with(|cs| {
        let mut guard = BUS.borrow(cs).borrow_mut();
        let bus = guard.as_mut().unwrap();
        bus.protocol.enqueue(packet);
        if !bus.reading && Instant::now() >= bus.rearm_at {
            bus.irq.set_low();
        }
    });
}

pub async fn run(keymap: &rmk::keymap::KeyMap<'_>) -> ! {
    loop {
        critical_section::with(|cs| {
            let mut guard = BUS.borrow(cs).borrow_mut();
            let bus = guard.as_mut().unwrap();
            bus.protocol.fn_lock = keymap.is_layer_active(2);
            if bus.protocol.pending() && !bus.reading && Instant::now() >= bus.rearm_at {
                bus.irq.set_low();
            }
        });
        Timer::after_millis(1).await;
    }
}
