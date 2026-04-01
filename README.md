# screen-flash

[中文 README](./README.zh.md)

A Rust library for Windows that displays a full-screen flash overlay.

The project provides two layers of usage:

- Library: implement or construct a `FlashEffect`
- CLI: choose a built-in effect and run a single flash

The core model is effect sampling:

- callers do not pass color and duration parameters directly
- each frame's color, opacity, and next sampling time are decided by `FlashEffect::sample`

## Features

- Full-screen flash overlay on Windows
- RGB color control
- Normalized alpha sampling in the range `0.0..=1.0`
- Variable step scheduling
- Time-varying color support

## Platform

- Windows only
- Built on Win32 APIs such as `CreateWindowExW`, `SetLayeredWindowAttributes`, and `RegisterClassW`
- Other platforms are not implemented yet. Contributions for macOS, Linux, and more are welcome

## Use As A Library

```toml
[dependencies]
screen-flash = { path = "." }
```

Replace `path` with a version number when publishing to crates.io.

## Quick Start

Using the built-in default effect:

```rust
use screen_flash::{DefaultFlashEffect, FlashColor, flash_screen};

fn main() -> windows::core::Result<()> {
    flash_screen(DefaultFlashEffect {
        color: FlashColor {
            red: 255,
            green: 255,
            blue: 255,
        },
    })
}
```

## Demo

Combined demo of:

- default effect with black color
- default effect with white color
- rainbow effect

![screen-flash demo](https://raw.githubusercontent.com/enximi/screen-flash/main/assets/demo/screen-flash-demo.gif)

## Core API

Library entry point:

```rust
pub fn flash_screen<E>(effect: E) -> Result<()>
where
    E: FlashEffect
```

Color type:

```rust
pub struct FlashColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}
```

Effect sample type:

```rust
pub struct FlashSample {
    pub color: FlashColor,
    pub alpha: f32,
    pub next_step_ms: Option<u64>,
}
```

Semantics:

- `color`: the color to use for the current frame
- `alpha`: the opacity for the current frame, in the range `0.0..=1.0`
- `next_step_ms`: delay before the next sample
- `Some(ms)`: continue after `ms` milliseconds
- `None`: the effect is finished

Effect trait:

```rust
pub trait FlashEffect {
    fn sample(&self, elapsed_ms: u64) -> FlashSample;
}
```

## Custom Effects

This example fades from red to transparent over 500ms:

```rust
use screen_flash::{FlashColor, FlashEffect, FlashSample, flash_screen};

struct RedFade;

impl FlashEffect for RedFade {
    fn sample(&self, elapsed_ms: u64) -> FlashSample {
        let duration_ms = 500;
        let finished = elapsed_ms >= duration_ms;
        let remaining = duration_ms.saturating_sub(elapsed_ms) as f32 / duration_ms as f32;

        FlashSample {
            color: FlashColor {
                red: 255,
                green: 0,
                blue: 0,
            },
            alpha: remaining.clamp(0.0, 1.0),
            next_step_ms: if finished { None } else { Some(16) },
        }
    }
}

fn main() -> windows::core::Result<()> {
    flash_screen(RedFade)
}
```

## Built-in Effects

The library currently includes two built-in effects:

- `DefaultFlashEffect`: customizable color, briefly holds a higher opacity, then fades out
- `RainbowFlashEffect`: longer duration, cycles through colors, and keeps a fixed opacity until it ends

`DefaultFlashEffect` behaves like this:

- starts with a short hold phase
- then fades out
- uses a default step size of `16ms`

The default effect does not restrict color. You provide it through `DefaultFlashEffect { color }`.

Rainbow example:

```rust
use screen_flash::{RainbowFlashEffect, flash_screen};

fn main() -> windows::core::Result<()> {
    flash_screen(RainbowFlashEffect)
}
```

## CLI

If you want to run the repository as a command-line tool:

```powershell
cargo run -- default
cargo run -- rainbow
```

For `default`, you can also pass a custom color:

```powershell
cargo run -- default --color ff6600
cargo run -- default --color 255,102,0
```

## Design Notes

The current abstraction merges animation and render state into one effect-sampling layer:

- it does not return only alpha
- it returns a complete visual state for each frame

That makes it natural to support:

- fixed-color flashes
- color transitions
- fixed opacity or fade-out effects
- variable step scheduling

## Tests

Run:

```powershell
cargo test
```
