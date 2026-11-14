use embassy_stm32::gpio::{Input, Output};
use embassy_time::{Duration, Instant, Ticker};
use rmk::event::KeyboardEvent;
use rmk::keyboard::Keyboard;

use crate::dispatch::dispatch;
use crate::ghost::ghosted;
use crate::i2c;
use crate::keymap::POSITIONS;

pub async fn run(mut rows: [Input<'static>; 8], mut cols: [Output<'static>; 18], keyboard: &mut Keyboard<'_>) -> ! {
    let mut held = [[false; 18]; 8];
    let mut previous = [0u32; 8];
    let mut changed_at = [[Instant::now(); 18]; 8];
    let mut ticker = Ticker::every(Duration::from_millis(1));
    loop {
        let mut raw = [0u32; 8];
        if i2c::scanning() {
            for (col, pin) in cols.iter_mut().enumerate() {
                pin.set_high();
                cortex_m::asm::delay(80); // 5 us settling at HSI16.
                for (row, input) in rows.iter_mut().enumerate() {
                    if input.is_high() {
                        raw[row] |= 1 << col;
                    }
                }
                pin.set_low();
            }
        }
        i2c::note_scan(raw);
        let now = Instant::now();
        for &(row, col) in &POSITIONS {
            let pressed = raw[row] & (1 << col) != 0;
            if (previous[row] ^ raw[row]) & (1 << col) != 0 {
                changed_at[row][col] = now;
            }
            let debounce = Duration::from_millis(if pressed { 1 } else { 5 });
            if pressed != held[row][col]
                && now - changed_at[row][col] >= debounce
                && (!pressed || !ghosted(&raw, row, col))
            {
                held[row][col] = pressed;
                i2c::note_event(row as u8, col as u8, pressed);
                dispatch(
                    keyboard,
                    KeyboardEvent::key(row as u8, col as u8, pressed),
                    i2c::submit_report,
                )
                .await;
            }
        }
        previous = raw;
        ticker.next().await;
    }
}
