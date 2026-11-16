# Dahlia (Framework 12 ANSI / ISO)

Minimal RMK firmware for the **DVT/MP Dahlia touchpad module**, STM32U073C8.
The matrix pins and key positions come from the Dahlia/Framework 12 ANSI
ZMK board files. The EVT pinout is not supported. ANSI is the default; build with
`--features iso` for the stock ISO matrix and extra keys.

Implemented:

- 8 × 18 active-high matrix, 1 ms press / 5 ms release debounce, ghost suppression.
- Inactive scan columns float to support chords on the diodeless membrane.
- ANSI or ISO typing, media/brightness keys, Fn navigation, persistent Fn lock.
- HID over I²C at `0x15`, IRQ on PA12 (open drain), PB6/PB7 at up to 1 MHz.
- IT8801 identification/register emulation at `0x38`; SOVR(2) bit 0 controls
  touchpad enable (PA15), bit 2 controls keyboard scanning. Disabling scanning
  releases held keys.
- Vendor firmware/hardware info (`framework_tool --stm32-kbd-info`), including
  the RMK Git commit, dirty-build flag, chip ID, memory sizes, and factory UID.
- BIOS Fn/Ctrl swap and masked vendor settings writes. Fn Lock, swap, update
  protection, and Caps Lock are queryable through vendor flags. Fn Lock survives
  power-off; Fn + Escape toggles it. Swap is reapplied by the EC at startup.
- Fn + Left Alt + T toggles the touchpad enable pin without disabling scanning.
- Media-launch key sends the stock consumer usage `0x0183`; Fn sends F12.
- Caps Lock indicator on PC13, controlled by the EC's HID LED reports.
- Fresh controllers clear `nBOOT_SEL` so the EC can enter the STM32 ROM
  bootloader via BOOT0 on subsequent power cycles. Three startup LED flashes
  indicate option-byte setup; five/seven/nine further flashes indicate failure.
- HID reset and state resynchronization; pending reports track the latest
  keyboard/media state so a slow EC cannot cause stale typing to replay.

Backlight and Copilot actions are handled by the EC through F15/F16; airplane
mode uses F17. Vial is omitted. The touchpad itself
continues to use its existing host connection; this firmware controls its enable
pin. The Caps Lock LED follows the host's state; it is not toggled locally.
The descriptor's engineering-firmware flag asks the BIOS to preserve this image.

## Build

From the RMK repository root:

```sh
nix develop
cd examples/use_rust/dahlia
cargo build --release
```

The ELF is `target/thumbv6m-none-eabi/release/rmk-dahlia`. To produce flash images, run:

```sh
cargo objcopy --release --bin rmk-dahlia -- -O ihex dahlia.hex
cargo objcopy --release --bin rmk-dahlia -- -O binary dahlia.bin
```

The linker limits code to `0x08000000..0x0800DFBF`, preserving the misc-flags
area. SRAM is 40 KiB. Fn Lock uses an append-only log in the final 8 KiB,
with one page erase per 256 changes and no writes for unchanged values.
This replaces ZMK storage; existing ZMK settings are not imported.

## Flash and bring up

From the `framework-system` checkout, flash the HEX and query the running image:

```sh
sudo ./target/debug/framework_tool --flash-stm32-kbd /path/to/rmk/examples/use_rust/dahlia/dahlia.hex
sudo ./target/debug/framework_tool --stm32-kbd-info
```

The firmware reports version 1.2.6 and the first eight bytes of the RMK Git
commit. The dirty flag indicates local changes at build time. Unknown vendor
commands return `0xff`. Vendor commands `0x10`/`0x11` set/read Fn/Ctrl swap;
`0x12`/`0x13` write/read flags. Bits 0 (Fn Lock) and 1 (swap) are writable;
bits 2 (update protection) and 3 (Caps Lock) are read-only. Flash failures return
status 2 and leave the requested settings unchanged.

Alternatively, connect an SWD probe and run from this directory:

```sh
probe-rs download --chip STM32U073C8 target/thumbv6m-none-eabi/release/rmk-dahlia
probe-rs reset --chip STM32U073C8
```

After resetting/re-enumerating the input deck, check normal typing, modifiers,
short taps, Fn + function/navigation keys, Fn + Escape, and EC keyboard/touchpad
controls. On Linux, check I²C HID enumeration in `dmesg` and key events with
`evtest`. The USB report queue is reused internally for RMK's wired routing;
there is no USB connection on this board. Each debounced matrix event is passed
directly to RMK's key processor, and its reports are submitted before processing
the next event. The board's plain-key/layer layout does not use timed behaviors.

Compilation and host-side protocol tests do not establish electrical or laptop
compatibility. Hardware bring-up is still required, particularly the I²C
clock-stretching/IRQ behavior and keyboard matrix settling.

## Host-side tests

Inside the Nix shell, from this directory:

```sh
cargo nextest run --lib --target x86_64-unknown-linux-gnu
```

These tests exercise the production dispatch path (immediate key reports and Fn
layer release), HID descriptors, slow-host coalescing, partial reads, reset
resynchronization, vendor info transactions, IT8801 controls, and ghost rectangles
without an MCU.
