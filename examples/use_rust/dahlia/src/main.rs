#![no_std]
#![no_main]

mod i2c;
mod matrix;

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, OutputOpenDrain, Pull, Speed};
use embassy_stm32::i2c::I2c;
use embassy_stm32::{Config, rcc};
use panic_halt as _;
use rmk::config::{BehaviorConfig, PositionalConfig};
use rmk::keyboard::Keyboard;
use rmk::types::connection::UsbState;
use rmk::{KeymapData, initialize_keymap};
use rmk_dahlia::{dispatch, ghost, keymap, protocol};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.hsi = true;
    config.rcc.msi = None;
    config.rcc.sys = rcc::Sysclk::HSI;
    let p = embassy_stm32::init(config);

    // Embassy configures the clocks and alternate-function pins; the board's
    // interrupt handler owns the peripheral from here on.
    let peripheral = I2c::new_blocking(p.I2C1, p.PB6, p.PB7, Default::default());
    core::mem::forget(peripheral);
    i2c::init(
        OutputOpenDrain::new(p.PA12, Level::High, Speed::VeryHigh),
        Output::new(p.PA15, Level::High, Speed::Low),
        Output::new(p.PC13, Level::Low, Speed::Low),
    );

    let rows = [
        Input::new(p.PB3, Pull::Down),
        Input::new(p.PB4, Pull::Down),
        Input::new(p.PB5, Pull::Down),
        Input::new(p.PA11, Pull::Down),
        Input::new(p.PA9, Pull::Down),
        Input::new(p.PA10, Pull::Down),
        Input::new(p.PB8, Pull::Down),
        Input::new(p.PB9, Pull::Down),
    ];
    let cols = [
        Output::new(p.PA0, Level::Low, Speed::Low),
        Output::new(p.PA1, Level::Low, Speed::Low),
        Output::new(p.PA8, Level::Low, Speed::Low),
        Output::new(p.PA2, Level::Low, Speed::Low),
        Output::new(p.PA3, Level::Low, Speed::Low),
        Output::new(p.PA4, Level::Low, Speed::Low),
        Output::new(p.PA5, Level::Low, Speed::Low),
        Output::new(p.PA6, Level::Low, Speed::Low),
        Output::new(p.PA7, Level::Low, Speed::Low),
        Output::new(p.PB0, Level::Low, Speed::Low),
        Output::new(p.PB1, Level::Low, Speed::Low),
        Output::new(p.PB2, Level::Low, Speed::Low),
        Output::new(p.PB10, Level::Low, Speed::Low),
        Output::new(p.PB11, Level::Low, Speed::Low),
        Output::new(p.PB12, Level::Low, Speed::Low),
        Output::new(p.PB13, Level::Low, Speed::Low),
        Output::new(p.PB14, Level::Low, Speed::Low),
        Output::new(p.PB15, Level::Low, Speed::Low),
    ];
    let mut data = KeymapData::new(keymap::default_keymap());
    let mut behavior = BehaviorConfig::default();
    let positional = PositionalConfig::default();
    let keymap = initialize_keymap(&mut data, &mut behavior, &positional).await;
    let mut keyboard = Keyboard::new(&keymap);
    // Reuse RMK's wired report queue; no USB peripheral is present on Dahlia.
    rmk::state::set_usb_state(UsbState::Configured);
    // This layout has plain keys and layer actions, with no timed behaviors.
    embassy_futures::join::join(matrix::run(rows, cols, &mut keyboard), i2c::run()).await;
}
