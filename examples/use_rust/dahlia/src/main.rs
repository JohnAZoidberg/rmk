#![no_std]
#![no_main]

mod i2c;
mod matrix;

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Flex, Input, Level, Output, OutputOpenDrain, Pull, Speed};
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
        Flex::new(p.PA0),
        Flex::new(p.PA1),
        Flex::new(p.PA8),
        Flex::new(p.PA2),
        Flex::new(p.PA3),
        Flex::new(p.PA4),
        Flex::new(p.PA5),
        Flex::new(p.PA6),
        Flex::new(p.PA7),
        Flex::new(p.PB0),
        Flex::new(p.PB1),
        Flex::new(p.PB2),
        Flex::new(p.PB10),
        Flex::new(p.PB11),
        Flex::new(p.PB12),
        Flex::new(p.PB13),
        Flex::new(p.PB14),
        Flex::new(p.PB15),
    ];
    let mut data = KeymapData::new(keymap::default_keymap());
    let mut behavior = BehaviorConfig::default();
    let positional = PositionalConfig::default();
    let keymap = initialize_keymap(&mut data, &mut behavior, &positional).await;
    let mut keyboard = Keyboard::new(&keymap);
    // Reuse RMK's wired report queue; no USB peripheral is present on Dahlia.
    rmk::state::set_usb_state(UsbState::Configured);
    // This layout has plain keys and layer actions, with no timed behaviors.
    embassy_futures::join::join(matrix::run(rows, cols, &mut keyboard), i2c::run(&keymap)).await;
}
