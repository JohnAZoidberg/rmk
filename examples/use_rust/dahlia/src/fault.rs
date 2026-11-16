//! Retain the stock BKP0R..BKP8R fault layout across a software reset.
use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

use cortex_m_rt::{ExceptionFrame, exception};
use embassy_stm32::pac;

const MAGIC: u32 = 0xdead_beef;
// TAMP's register layout is absent from the PAC; BKP0R is at offset 0x100.
const BACKUP: *mut u32 = 0x4000_b100 as *mut u32;

fn enable_access() {
    pac::RCC.apbenr1().modify(|w| {
        w.set_pwren(true);
        w.set_rtcapben(true);
    });
    let _ = pac::RCC.apbenr1().read();
    pac::PWR.cr1().modify(|w| w.set_dbp(true));
    while !pac::PWR.cr1().read().dbp() {}
}

pub fn present() -> bool {
    enable_access();
    // SAFETY: backup-domain access is enabled and BKP0R is a valid register.
    unsafe { read_volatile(BACKUP) == MAGIC }
}

#[exception]
unsafe fn HardFault(frame: &ExceptionFrame) -> ! {
    capture(1, Some(frame))
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    capture(2, None)
}

fn capture(reason: u32, frame: Option<&ExceptionFrame>) -> ! {
    cortex_m::interrupt::disable();
    enable_access();
    // SAFETY: all nine registers exist on STM32U073; interrupts are disabled.
    unsafe {
        write_volatile(BACKUP, 0);
        write_volatile(BACKUP.add(1), reason);
        write_volatile(BACKUP.add(2), frame.map_or(0, ExceptionFrame::pc));
        write_volatile(BACKUP.add(3), frame.map_or(0, ExceptionFrame::lr));
        write_volatile(BACKUP.add(4), frame.map_or(0, ExceptionFrame::xpsr));
        write_volatile(BACKUP.add(5), frame.map_or(0, ExceptionFrame::r0));
        write_volatile(BACKUP.add(6), frame.map_or(0, ExceptionFrame::r3));
        write_volatile(BACKUP.add(7), frame.map_or(0, ExceptionFrame::r12));
        write_volatile(BACKUP.add(8), read_volatile(0xe000_ed04 as *const u32));
        // Publish the record only after every field is written.
        write_volatile(BACKUP, MAGIC);
    }
    cortex_m::asm::dsb();
    cortex_m::peripheral::SCB::sys_reset()
}
