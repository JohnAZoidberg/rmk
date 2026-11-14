// Copyright 2022 Framework Computer
// SPDX-License-Identifier: GPL-2.0-or-later
// ADC scanning algorithm derived from Framework's QMK matrix.c.
use crate::keymap::{COL, ROW};
use embassy_rp::{
    adc::{Adc, Channel},
    gpio::{Input, Output},
    mode::Blocking,
};
use embassy_time::Timer;
use rmk::{
    core_traits::Runnable,
    debounce::{DebounceState, DebouncerTrait, default_debouncer::DefaultDebouncer},
    event::{KeyboardEvent, publish_event_async},
    matrix::KeyState,
};

pub struct FrameworkMatrix {
    pub adc: Adc<'static, Blocking>,
    pub channel: Channel<'static>,
    pub mux: [Output<'static>; 3],
    pub columns: [Output<'static>; COL],
    pub lid: Input<'static>,
}

impl Runnable for FrameworkMatrix {
    async fn run(&mut self) -> ! {
        let mut states = [[KeyState::new(); COL]; ROW];
        let mut debounce = DefaultDebouncer::<ROW, COL>::new();
        loop {
            let mut voltages = [[33000u32; COL]; ROW];
            if self.lid.is_high() {
                let mut valid = true;
                for col in 0..COL {
                    self.columns[col].set_low();
                    for row in 0..ROW {
                        let index = match row {
                            0 => 2,
                            1 => 0,
                            2 => 1,
                            n => n,
                        };
                        for (bit, pin) in self.mux.iter_mut().enumerate() {
                            pin.set_level((index & (1 << bit) != 0).into());
                        }
                        Timer::after_micros(2).await;
                        match self.adc.blocking_read(&mut self.channel) {
                            Ok(sample) => voltages[row][col] = u32::from(sample) * 33000 / 4095,
                            Err(_) => valid = false,
                        }
                    }
                    self.columns[col].set_high();
                }
                if !valid {
                    Timer::after_millis(1).await;
                    continue;
                }
            }
            // A closed lid produces releases too, so reopening cannot leave stuck keys.
            for row in 0..ROW {
                let count = voltages[row].iter().filter(|v| **v < 29000).count();
                let threshold = match count {
                    0 | 1 => 10000,
                    2 => 20000,
                    3 => 25000,
                    _ => 29000,
                };
                for col in 0..COL {
                    if let DebounceState::Debounced = debounce.detect_change_with_debounce(
                        row,
                        col,
                        voltages[row][col] < threshold,
                        &states[row][col],
                    ) {
                        states[row][col].toggle_pressed();
                        publish_event_async(KeyboardEvent::key(
                            row as u8,
                            col as u8,
                            states[row][col].pressed,
                        ))
                        .await;
                    }
                }
            }
            Timer::after_micros(100).await;
        }
    }
}
