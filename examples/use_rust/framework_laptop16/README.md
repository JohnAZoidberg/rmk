# Framework Laptop 16 input modules

RMK firmware for RP2040-based Framework Laptop 16 ANSI, ISO and JIS keyboards,
the 21-key numpad, and the 24-key RGB macropad. Matrix wiring and default keymaps
come from `keyboards/framework` in Framework's QMK firmware (GPL-2.0-or-later).
This port is also GPL-2.0-or-later.

Build from this directory with the RMK development environment:

```sh
cargo build --release                                      # ANSI
cargo build --release --no-default-features --features iso
cargo build --release --no-default-features --features jis
cargo build --release --no-default-features --features numpad
cargo build --release --no-default-features --features macropad
```

Convert `target/thumbv6m-none-eabi/release/rmk-framework-laptop16` to UF2 with
`elf2uf2-rs`. Enter the RP2040 ROM bootloader by holding both Alt keys while
installing a keyboard, or 2 and 6 while installing a numpad. Use the hardware
BOOTSEL/reset mechanism for a macropad. Copy the UF2 to the `RPI-RP2` drive.

The scanner uses GPIO1–3 to select the mux, GPIO4 low to enable it, GPIO28
(ADC2) with a pull-up to read it, and the original KSO column ordering.
The mux row mapping is 2, 0, 1, 3, 4, 5, 6, 7. Adaptive per-row thresholds
match QMK's 1.0/2.0/2.5/2.9 V thresholds, scaled for the RP2040's 12-bit ADC.
RMK debounces the resulting states. GPIO0 low suppresses keys and releases
held keys. GPIO5 low marks boot complete. GPIO24 shows host Caps Lock on
keyboards and Num Lock on the numpad.

Keyboard Fn layers include media keys, navigation and persistent Fn lock
(Fn+Escape). Numpad/macropad use the QMK default keypad map; the host handles
Num Lock navigation. Flash storage occupies 956–1020 KiB, leaving the final
4 KiB factory serial sector intact. The linker excludes both areas.
Flashing this firmware replaces the existing QMK configuration in that storage
area; it does not migrate it.

## Current limitations

This is a keyboard-input port pending physical hardware validation. Backlights
and the IS31FL3743A RGB controller are held off. Lighting actions and airplane
mode are unassigned. Framework factory/BIOS vendor commands, factory serial
number descriptors, automatic BIOS hotkey-mode switching, and Vial are not
implemented. USB uses RMK's chip-derived serial number. GPIO0 gating is
implemented, but lid and USB suspend/wake behavior needs hardware verification.
The analog settling interval and simultaneous-key thresholds also need testing
on each physical module.
