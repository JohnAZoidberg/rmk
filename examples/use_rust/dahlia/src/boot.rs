//! Keep the EC's BOOT0 power-cycle sequence available on fresh controllers.
use embassy_stm32::gpio::Output;
use embassy_stm32::pac;

pub fn enable_boot0(led: &mut Output<'_>) {
    let flash = pac::FLASH;
    if !flash.optr().read().nboot_sel() {
        return;
    }
    blink(led, 3);
    let result = critical_section::with(|_| {
        while flash.sr().read().bsy1() {}
        flash.sr().write_value(flash.sr().read());
        flash.keyr().write(|w| w.set_key(0x4567_0123));
        flash.keyr().write(|w| w.set_key(0xcdef_89ab));
        if flash.cr().read().lock() {
            return Err(5);
        }
        flash.optkeyr().write(|w| w.set_optkey(0x0819_2a3b));
        flash.optkeyr().write(|w| w.set_optkey(0x4c5d_6e7f));
        if flash.cr().read().optlock() {
            flash.cr().modify(|w| w.set_lock(true));
            return Err(7);
        }
        flash.optr().modify(|w| w.set_nboot_sel(false));
        flash.cr().modify(|w| w.set_optstrt(true));
        while flash.sr().read().bsy1() {}
        flash.cr().modify(|w| {
            w.set_optstrt(false);
            w.set_optlock(true);
            w.set_lock(true);
        });
        let status = flash.sr().read();
        if status.progerr() || status.wrperr() || status.pgaerr() || status.optverr() {
            Err(9)
        } else {
            Ok(())
        }
    });
    // As in stock firmware, the option takes effect on the next power cycle.
    if let Err(blinks) = result {
        blink(led, blinks);
    }
}

fn blink(led: &mut Output<'_>, count: usize) {
    for _ in 0..count {
        led.set_high();
        cortex_m::asm::delay(3_200_000);
        led.set_low();
        cortex_m::asm::delay(3_200_000);
    }
}
