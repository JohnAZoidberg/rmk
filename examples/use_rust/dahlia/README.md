# Dahlia (Framework 12 ANSI / ISO)

RMK firmware for the **DVT/MP Dahlia touchpad module**, STM32U073C8.
The matrix pins and key positions come from the Dahlia/Framework 12 ANSI/ISO
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
- HardFault and panic capture in TAMP backup registers, followed by reset. A
  retained record overrides the Caps Lock LED with three short flashes and one
  long flash while the keyboard continues operating.
- HID reset and state resynchronization; pending reports track the latest
  keyboard/media state so a slow EC cannot cause stale typing to replay.

Backlight and Copilot actions are handled by the EC through F15/F16; airplane
mode uses F17. Vial is omitted. The touchpad itself continues to use its existing host connection; this firmware controls its enable
pin. The Caps Lock LED follows the host unless a retained fault is being signaled.
By default the HID descriptor asks the BIOS to preserve this image. Building with
`--features allow-auto-update` opts into BIOS replacement, unless stock's
misc-flags slot 0 at `0x0800DFC0` is all zero. That flash slot protects the image
regardless of build policy. `GET_FLAGS` and the descriptor report the same state;
this flag does not block explicit EC flashing.

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

The firmware reports version 1.2.8 and the first eight bytes of the RMK Git
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
cargo nextest run --lib --target x86_64-unknown-linux-gnu --features iso,allow-auto-update
```

These tests exercise the production dispatch path (immediate key reports and Fn
layer release), HID descriptors, slow-host coalescing, partial reads, reset
resynchronization, vendor settings and info transactions, IT8801 controls, touchpad
shortcuts, ISO keys, persistent Fn Lock, flash-log rotation, and ghost rectangles
without an MCU. MCU-only fault and option-byte paths require the hardware checks
below.

## Fault diagnostics

BKP0R..BKP8R at `0x4000B100..0x4000B120` retain the stock record layout:
magic `0xDEADBEEF`, reason, PC, LR, xPSR, R0, R3, R12, and ICSR. RMK reason 1
means HardFault; reason 2 means panic, whose exception-frame fields are zero.
These are backup registers, not flash: retention is across software reset and
depends on backup-domain power. The register offsets match the
[STM32U073 CMSIS header](https://github.com/STMicroelectronics/cmsis-device-u0/blob/main/Include/stm32u073xx.h).

Build with `--features fault-test` only for diagnostics. On its first boot with
no retained record it executes UDF, captures the HardFault, and resets; subsequent
boots blink the fault pattern without faulting again. Read the nine words over
SWD, then restore a normal build. With backup-domain access enabled, clear BKP0R
and reset to restore the normal Caps Lock LED. The `fault-test` build faults again
if its record is cleared.

## Hardware acceptance checklist

Use the normal ANSI image first; use the ISO image only with an ISO keyboard.
The EC flashing tool mass-erases the controller, so flashing clears saved Fn Lock
and misc-flags. Test persistence with a reset or deck power cycle, without
reflashing between checks.

1. Flash the normal image and run `framework_tool --stm32-kbd-info`. Expect
   version 1.2.8, the build's commit hash, chip ID `0x0489`, 64 KiB flash,
   40 KiB RAM, and do-not-update enabled. Check typing and modifier chords.
   Run the info command repeatedly using Enter, then confirm Enter releases and
   typing continues normally; the vendor query must not strand keyboard input.
2. Toggle Fn Lock with Fn+Esc, confirm the reported state, then run
   `framework_tool --stm32-kbd-reset`. Confirm Fn Lock remains on and one
   Fn+Esc chord turns it off. Repeat with a full deck power cycle.
3. Run `framework_tool --stm32-kbd-fn-lock on`, confirm the function row changes
   and `--stm32-kbd-info` reports it. Repeat `on`, reset, then test `off`.
   Fn Lock must not change Space/Delete behavior without Fn held.
4. Change the BIOS Fn/Ctrl swap setting and let the deck re-enumerate. Confirm
   the physical keys exchange roles, including Fn+Esc and Fn while locked.
   Confirm `--stm32-kbd-info` reports the new swap state. Restore the BIOS setting.
5. Check Caps Lock's LED and reported state. Verify media/brightness keys, the
   media-launch key (`0x0183`), Fn+F12, and Fn+K/P/B. Check Fn+Space backlight,
   Fn+Right Ctrl Copilot, and the airplane key through the EC.
6. Press Fn+Left Alt+T twice. Only the touchpad should toggle; typing continues.
   Hold T and change other keys to confirm it does not toggle repeatedly.
   Check lid/tablet/suspend controls still disable and restore the deck correctly,
   including while keys are held.
7. With an ISO keyboard and `--features iso`, check the extra key beside Left
   Shift and the key beside Enter, on base/Fn/Fn Lock/temporary-unlock layers.
8. On a fresh controller, watch for the three option-setup flashes. Power-cycle,
   then use `framework_tool --stm32-kbd-identify` to exercise ROM bootloader entry
   and return to normal typing. A controller previously running stock may already
   have `nBOOT_SEL` cleared and will skip the startup flashes.
9. To test flash-slot protection, build with `--features allow-auto-update`.
   Flash normally and expect do-not-update off. Reflash with
   `--stm32-kbd-update-protect` and expect it on. Restore the normal protected build.
10. With SWD available, use the `fault-test` build and the fault-diagnostics steps
    above. Verify one reset, the retained PC/ICSR, the three-short/one-long LED
    pattern, and continued typing. Restore a normal build and clear the record.

Commands above require the same privileges as the flashing command. Record which
checks passed on physical hardware separately from the host-test results.
