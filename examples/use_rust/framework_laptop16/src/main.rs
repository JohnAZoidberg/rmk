#![no_main]
#![no_std]
#[cfg(feature = "ansi")]
#[path = "ansi.rs"]
mod keymap;
#[cfg(feature = "iso")]
#[path = "iso.rs"]
mod keymap;
#[cfg(feature = "jis")]
#[path = "jis.rs"]
mod keymap;
#[cfg(feature = "numpad")]
#[path = "numpad.rs"]
mod keymap;
#[cfg(feature = "macropad")]
#[path = "macropad.rs"]
mod keymap;
mod matrix;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel};
use embassy_rp::flash::Flash;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::mode::Async;
use embassy_rp::peripherals::{DMA_CH0, USB};
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::{bind_interrupts, dma};

use panic_probe as _;
use rmk::config::{BehaviorConfig, DeviceConfig, PositionalConfig, RmkConfig, StorageConfig};

use rmk::keyboard::Keyboard;

use rmk::processor::builtin::wpm::WpmProcessor;
use rmk::usb::UsbTransport;
use rmk::watchdog::Rp2040Watchdog;
use rmk::{KeymapData, initialize_keymap_and_storage, run_all};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>;
});

const FLASH_SIZE: usize = 1024 * 1024;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("RMK start!");
    // Initialize peripherals
    let mut p = embassy_rp::init(Default::default());
    let serial_number = rmk::config::rp2040_serial_number(p.FLASH.reborrow());

    // Create the usb driver, from the HAL
    let driver = Driver::new(p.USB, Irqs);

    let _boot_done = Output::new(p.PIN_5, Level::Low);
    let _mux_enable = Output::new(p.PIN_4, Level::Low);
    let _rgb_disable = Output::new(p.PIN_29, Level::Low);
    let _backlight_off = Output::new(p.PIN_25, Level::Low);
    let _unused_rows = (
        Input::new(p.PIN_6, Pull::None),
        Input::new(p.PIN_7, Pull::None),
    );
    let columns = [
        Output::new(p.PIN_8, Level::High),
        Output::new(p.PIN_9, Level::High),
        Output::new(p.PIN_10, Level::High),
        Output::new(p.PIN_11, Level::High),
        Output::new(p.PIN_12, Level::High),
        Output::new(p.PIN_13, Level::High),
        Output::new(p.PIN_14, Level::High),
        Output::new(p.PIN_15, Level::High),
        #[cfg(any(feature = "ansi", feature = "iso", feature = "jis"))]
        Output::new(p.PIN_21, Level::High),
        #[cfg(any(feature = "ansi", feature = "iso", feature = "jis"))]
        Output::new(p.PIN_20, Level::High),
        #[cfg(any(feature = "ansi", feature = "iso", feature = "jis"))]
        Output::new(p.PIN_19, Level::High),
        #[cfg(any(feature = "ansi", feature = "iso", feature = "jis"))]
        Output::new(p.PIN_18, Level::High),
        #[cfg(any(feature = "ansi", feature = "iso", feature = "jis"))]
        Output::new(p.PIN_17, Level::High),
        #[cfg(any(feature = "ansi", feature = "iso", feature = "jis"))]
        Output::new(p.PIN_16, Level::High),
        #[cfg(any(feature = "ansi", feature = "iso", feature = "jis"))]
        Output::new(p.PIN_23, Level::High),
        #[cfg(any(feature = "ansi", feature = "iso", feature = "jis"))]
        Output::new(p.PIN_22, Level::High),
    ];
    let mut matrix = matrix::FrameworkMatrix {
        adc: Adc::new_blocking(p.ADC, Default::default()),
        channel: Channel::new_pin(p.PIN_28, Pull::Up),
        mux: [
            Output::new(p.PIN_1, Level::Low),
            Output::new(p.PIN_2, Level::Low),
            Output::new(p.PIN_3, Level::Low),
        ],
        columns,
        lid: Input::new(p.PIN_0, Pull::None),
    };

    // Use internal flash to emulate eeprom
    let flash = Flash::<Async, FLASH_SIZE>::new(p.FLASH, p.DMA_CH0, Irqs);

    let keyboard_device_config = DeviceConfig {
        serial_number,
        vid: 0x32ac,
        pid: keymap::PID,
        manufacturer: "Framework",
        product_name: "Framework Laptop 16 RMK",
        ..DeviceConfig::default()
    };

    let rmk_config = RmkConfig {
        device_config: keyboard_device_config,
        ..Default::default()
    };

    // Initialize the storage and keymap
    let mut keymap_data = KeymapData::new(keymap::get_default_keymap());
    let storage_config = StorageConfig {
        start_addr: 956 * 1024,
        num_sectors: 16,
        ..Default::default()
    };
    let mut behavior_config = BehaviorConfig::default();
    let per_key_config = PositionalConfig::default();
    let (keymap, mut storage) = initialize_keymap_and_storage(
        &mut keymap_data,
        flash,
        &storage_config,
        &mut behavior_config,
        &per_key_config,
    )
    .await;

    // Initialize the matrix + keyboard
    let mut keyboard = Keyboard::new(&keymap);

    let mut usb_transport = UsbTransport::new(driver, rmk_config.device_config);
    let mut wpm_processor = WpmProcessor::new();

    let mut watchdog_runner =
        Rp2040Watchdog::default_runner(embassy_rp::watchdog::Watchdog::new(p.WATCHDOG));

    #[cfg(feature = "numpad")]
    let indicator_type = rmk::types::led_indicator::LedIndicatorType::NumLock;
    #[cfg(any(feature = "ansi", feature = "iso", feature = "jis"))]
    let indicator_type = rmk::types::led_indicator::LedIndicatorType::CapsLock;
    #[cfg(not(feature = "macropad"))]
    let mut indicator = Some(
        rmk::processor::builtin::led_indicator::KeyboardIndicatorProcessor::new(
            Output::new(p.PIN_24, Level::Low),
            false,
            indicator_type,
        ),
    );
    #[cfg(feature = "macropad")]
    let mut indicator: Option<
        rmk::processor::builtin::led_indicator::KeyboardIndicatorProcessor<Output<'_>>,
    > = None;
    // Start
    run_all!(
        matrix,
        storage,
        usb_transport,
        wpm_processor,
        keyboard,
        watchdog_runner,
        indicator
    )
    .await;
}
