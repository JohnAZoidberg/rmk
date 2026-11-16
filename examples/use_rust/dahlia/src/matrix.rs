use embassy_stm32::gpio::{Flex, Input, Pull, Speed};
use embassy_time::{Duration, Instant, Ticker};
use rmk::event::KeyboardEvent;
use rmk::keyboard::Keyboard;

use crate::dispatch::dispatch;
use crate::ghost::ghosted;
use crate::i2c;
use crate::keymap::POSITIONS;

pub async fn run(
    mut rows: [Input<'static>; 8],
    mut cols: [Flex<'static>; 18],
    keyboard: &mut Keyboard<'_>,
    keymap: &rmk::keymap::KeyMap<'_>,
) -> ! {
    for pin in &mut cols {
        pin.set_as_input(Pull::None);
    }
    let mut swap = crate::dispatch::FnCtrlSwap::default();
    let mut held = [[false; 18]; 8];
    let mut previous = [0u32; 8];
    let mut changed_at = [[Instant::now(); 18]; 8];
    let mut ticker = Ticker::every(Duration::from_millis(1));
    loop {
        keymap.set_layer_active(2, i2c::fn_lock());
        let mut raw = [0u32; 8];
        if i2c::scanning() {
            for (col, pin) in cols.iter_mut().enumerate() {
                // Inactive columns must float: this membrane has no diodes.
                pin.set_high();
                pin.set_as_output(Speed::Low);
                cortex_m::asm::delay(80); // 5 us settling at HSI16.
                for (row, input) in rows.iter_mut().enumerate() {
                    if input.is_high() {
                        raw[row] |= 1 << col;
                    }
                }
                pin.set_as_input(Pull::None);
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
                let (row, col) = swap.position(row, col, pressed, i2c::fn_ctrl_swap());
                let previous_lock = i2c::fn_lock();
                keymap.set_layer_active(2, previous_lock);
                dispatch(
                    keyboard,
                    KeyboardEvent::key(row as u8, col as u8, pressed),
                    i2c::submit_report,
                )
                .await;
                let locked = keymap.is_layer_active(2);
                if locked != previous_lock && !i2c::save_fn_lock(locked) {
                    keymap.set_layer_active(2, i2c::fn_lock());
                }
            }
        }
        previous = raw;
        ticker.next().await;
    }
}
