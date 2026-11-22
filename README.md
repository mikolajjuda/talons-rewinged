# Talons Rewinged

A Rust rewrite of Talons.HC from TempleOS. All sprites (bird, fish, boats, trees, claws) are read at startup from the original `Talons.HC`.
Fly a bird over generated terrain and catch ten fish by flying low over the water. Crashing into land or leaving the map ends the run.
`Enter` starts a new world.

## Build

```sh
cargo build --release
cargo build --release --target x86_64-pc-windows-gnu
```

Linux needs the SDL2 and ALSA development packages. The Windows cross-build needs `mingw-w64` and `cmake`.

Run the executable from the folder where `input.json` and `best_times.txt` should live.

## Modes

**Classic Game** – no physics. The bird always flies along its nose. Speed is a single number: it grows when diving, drains when climbing or inverted, and drops to a slow glide near the water. There is no gravity, lift, drag or stalling; pitch and roll rotate the nose directly. Controls: pitch, roll.

**Bird Physics** – simulated flight: gravity, lift, drag and stalls. Control surfaces apply torque that is damped, so turns need roll and pitch together. Throttle flaps the wings for thrust, and the air brake spreads them to slow down. Controls: pitch, roll, yaw, throttle, air brake.

**Drone Acro (FPV)** – an acro-mode quadcopter. Sticks set rotation rates (no self-levelling), throttle is thrust along the drone's up axis, and only gravity and drag act on it, so you must hold throttle to stay in the air. Controls: pitch, roll, yaw, throttle.

## Controls

| Action | Key |
| --- | --- |
| Pitch | `W` / `S` |
| Roll | `A` / `D` |
| Yaw | `Q` / `E` |
| Throttle | `Left Shift` |
| Air brake | `Space` |
| New world | `Enter` |
| Menu (quit from menu) | `Esc` |
| Mute | `M` |
| FPS counter | `F` |
| Fullscreen | `F11` or `Alt+Enter` |

### Custom bindings and gamepads

Copy `input.example.json` to `input.json` and edit it. Keyboard defaults stay active, so a gamepad setup is additive. The Input Test screen in the menu lists the key, axis and button names of every connected device and reloads `input.json`.

* `pad`: `"Keyboard"`, part of a gamepad name (e.g. `"xbox"`), or `"*"` for any pad.
* `axis`: gamepad axis name such as `"Axis0"`, or `null` to use `neg` / `pos` keys.
* `raw` / `out`: input range reported by the device / range sent to the game (swap `out` values to invert).
* `deadzone`: inputs smaller than this count as zero.
* `switch` (air brake): `button` such as `"Space"` or `"Button0"`, or `axis` with `threshold`, plus `invert`.

Example, left stick for roll and pitch:

```json
"roll":  { "pad": "*", "axis": "Axis2", "raw": [-1.0, 1.0], "out": [-1.0, 1.0], "deadzone": 0.1 },
"pitch": { "pad": "*", "axis": "Axis3", "raw": [-1.0, 1.0], "out": [1.0, -1.0], "deadzone": 0.1 }
```
