# `stm32f1xx-hal`

> [HAL] for the STM32F1 family of microcontrollers

[HAL]: https://crates.io/crates/embedded-hal

[![Crates.io](https://img.shields.io/crates/d/stm32f1xx-hal.svg)](https://crates.io/crates/stm32f1xx-hal)
[![Crates.io](https://img.shields.io/crates/v/stm32f1xx-hal.svg)](https://crates.io/crates/stm32f1xx-hal)
[![Released API docs](https://docs.rs/stm32f1xx-hal/badge.svg)](https://docs.rs/stm32f1xx-hal)
[![dependency status](https://deps.rs/repo/github/stm32-rs/stm32f1xx-hal/status.svg)](https://deps.rs/repo/github/stm32-rs/stm32f1xx-hal)
[![Continuous integration](https://github.com/stm32-rs/stm32f1xx-hal/workflows/Continuous%20integration/badge.svg)](https://github.com/stm32-rs/stm32f1xx-hal)

## Quick start guide

Embedded Rust development requires a bit more setup than ordinary development.
For this guide, we'll assume you're using a stm32 blue pill board (shown
below), but if you have another f1 microcontroller, you should be able to adapt
it.
嵌入式 Rust 开发比普通开发需要更多的设置。 在本指南中，我们假设您使用的是 stm32 蓝色药丸板（如图所示 ），但如果你有另一个 F1 微控制器，你应该能够适应 它。

![blue pill pinout](BluePillPinout.jpg "opt title")

You will also need a debug probe, for example an [stlink v3
mini](https://www.st.com/en/development-tools/stlink-v3mini.html) for programming and debugging.
(There are many different STLink probes out there, all of them _should_ work fine with the instructions given here, other JTAG or SWD debug probes will work as well but will need different software or configuration).

您还需要一个调试探针，例如 stlink v3 mini 用于编程和调试。 （市面上有许多不同的 STLink 探针，它们都应该按照这里给出的说明正常工作，其他 JTAG 或 SWD 调试探针也可以工作，但需要不同的软件或配置）。

### Installing software

To program your microcontroller, you need to install:
- [openocd](http://openocd.org/)
- `gdb-multiarch` (on some platforms you may need to use `gdb-arm-none-eabi` instead, make sure to update `.cargo/config` to reflect this change)
gdb-multiarch（在某些平台上，您可能需要改用，请务必更新以反映此更改）gdb-arm-none-eabi.cargo/config

Finally, you need to install arm target support for the Rust compiler. To do
so, run
最后，您需要为 Rust 编译器安装 arm 目标支持。待办事项 所以，运行
```
rustup target install thumbv7m-none-eabi
```


### Setting up your project

Create a new Rust project as you usually do with `cargo init`. The hello world
of embedded development is usually to blink an LED and code to do so is
available in [examples/blinky.rs](examples/blinky.rs). Copy that file to the
`main.rs` of your project.
像通常使用 .hello world 的嵌入式开发通常是闪烁 LED，而这样做的代码是 在 examples/blinky.rs 中可用。将该文件复制到您的项目中。

You also need to add some dependencies to your `Cargo.toml`:

```toml
[dependencies]
embedded-hal = "0.2.7"
nb = "1"
cortex-m = "0.7.6"
cortex-m-rt = "0.7.1"
# Panic behaviour, see https://crates.io/keywords/panic-impl for alternatives
panic-halt = "0.2.0"

[dependencies.stm32f1xx-hal]
version = "0.10.0"
features = ["rt", "stm32f103", "medium"]
```

If you build your project now, you should get a single error: `error: language
item required, but not found: eh_personality`. This unhelpful error message 
is fixed by compiling for the right target.
如果您现在构建项目，您应该会收到一个错误：.此无用错误消息 通过针对正确的目标进行编译来修复

We also need to tell Rust how to link our executable, and how to lay out the
result in memory. To accomplish all this, copy
[.cargo/config.toml](.cargo/config.toml) and [memory.x](memory.x) from the
stm32f1xx-hal repo to your project.
我们还需要告诉 Rust 如何链接我们的可执行文件，以及如何布置 结果。要完成所有这些作，请从 stm32f1xx-hal 存储库添加到您的项目中。

```bash
cargo build
```

If everything went well, your project should have built without errors.
如果一切顺利，您的项目应该构建没有错误。

### Programming the microcontroller

It is now time to actually run the code on the hardware.  To do so plug your
debug probe into the blue pill and start `openocd` using
现在是时候在硬件上实际运行代码了。为此，请将您的 debug 探针到蓝色药丸中并开始使用openocd
```bash
openocd -f interface/stlink-v3.cfg -f target/stm32f1x.cfg
```
If you are not using an stlink V3, change the interface accordingly. 
For more information, see the [embeddonomicon].
如果您不使用 stlink V3，请相应地更改接口。 有关更多信息，请参阅 embeddonomicon。

If all went well, it should detect your microcontroller and say `Info :
stm32f1x.cpu: hardware has 6 breakpoints, 4 watchpoints`. Keep it running in
the background.
如果一切顺利，它应该会检测到您的微控制器并说 。保持运行 背景。

We will use gdb for uploading the compiled binary to the microcontroller and
for debugging. Cargo will automatically start `gdb` thanks to the
[.cargo/config](.cargo/config) you added earlier. `gdb` also needs to be told
to connect to openocd which is done by copying [.gdbinit](.gdbinit) to the root
of your project.
我们将使用 gdb 将编译后的二进制文件上传到微控制器，并使用 用于调试。Cargo 将自动启动，这要归功于您之前添加的 .cargo/config。 也需要告诉 连接到 OpenOCD，这是通过将 .gdbinit 复制到根来完成的 的 Projects。

You may also need to tell `gdb` that it is safe to load `.gdbinit` from the
working directory.
您可能还需要告诉从 working 目录中。
- Linux
  ```bash
  echo "set auto-load safe-path $(pwd)" >> ~/.gdbinit
  ```
- Windows
  ```batch
  echo set auto-load safe-path %CD% >> %USERPROFILE%\.gdbinit
  ```
  *You may need restart your computer*

If everything was successful, cargo should compile your project, start gdb,
load your program and give you a prompt. If you type `continue` in the gdb
prompt, your program should start and the green led on the blue pill should
start blinking.
如果一切成功，cargo 应该编译你的项目，启动 gdb， 加载您的程序并为您提供提示。如果您在 gdb 中键入 提示，您的程序应启动，蓝色药丸上的绿色 LED 应 开始闪烁。


### Going further

From here on, you can start adding more code to your project to make it do
something more interesting. For crate documentation, see
[docs.rs/stm32f1xx-hal](https://docs.rs/stm32f1xx-hal). There are also a lot
more [examples](examples) available. If something is unclear in the docs or
examples, please, open an issue and we will try to improve it.
从这里开始，您可以开始向项目添加更多代码以使其完成 更有趣的东西。有关 crate 文档，请参阅 docs.rs/stm32f1xx-hal。也有很多 更多示例可用。如果文档中的内容不清楚，或者 examples，请打开一个问题，我们将尝试改进它。



## Selecting a microcontroller

This crate supports multiple microcontrollers in the
stm32f1 family. Which specific microcontroller you want to build for has to be
specified with a feature, for example `stm32f103`. 
此 crate 支持 STM32F1 系列。您要构建的特定微控制器必须是 使用特征指定，例如 .stm32f103

If no microcontroller is specified, the crate will not compile.
如果未指定 microcontroller，则 crate 将不会编译。

You may also need to specify the density of the device with `medium`, `high` or `xl` 
to enable certain peripherals. Generally the density can be determined by the 2nd character 
after the number in the device name (i.e. For STM32F103C6U, the 6 indicates a low-density
device) but check the datasheet or CubeMX to be sure.
您可能还需要使用 指定设备的密度，或启用某些外围设备。通常，密度可以由第 2 个字符确定 在设备名称中的数字之后（即 For STM32F103C6U，6 表示低密度 device），但请检查数据表或 CubeMX 以确保。
* 4, 6 => low density, no feature required
* 8, B => `medium` feature
* C, D, E => `high` feature
* F, G => `xl` feature

For microcontrollers of the `connectivity line` (`stm32f105` and `stm32f107`) no
density feature must be specified.

### Supported Microcontrollers

* `stm32f100`
* `stm32f101`
* `stm32f103`
* `stm32f105`
* `stm32f107`

## Trying out the examples

You may need to give `cargo` permission to call `gdb` from the working directory.
您可能需要授予从工作目录调用 的权限
- Linux
  ```bash
  echo "set auto-load safe-path $(pwd)" >> ~/.gdbinit
  ```
- Windows
  ```batch
  echo set auto-load safe-path %CD% >> %USERPROFILE%\.gdbinit
  ```
  *You may need restart your computer*
  Windows 您可能需要重新启动计算机

Compile, load, and launch the hardware debugger.
编译、加载并启动硬件调试器。
```bash
$ rustup target add thumbv7m-none-eabi

# on another terminal
$ openocd -f interface/$INTERFACE.cfg -f target/stm32f1x.cfg

# flash and debug the "Hello, world" example. Change stm32f103 to match your hardware
$ cargo run --features stm32f103 --example hello
```

`$INTERFACE` should be set based on your debugging hardware. If you are using
an stlink V2, use `stlink-v2.cfg`. For more information, see the
[embeddonomicon].
$INTERFACE应根据您的调试硬件进行设置。如果您正在使用 一个 stlink V2，使用 .有关更多信息，请参阅 embeddonomicon。

[embeddonomicon]: https://rust-embedded.github.io/book/start/hardware.html



## Using as a Dependency

When using this crate as a dependency in your project, the microcontroller can 
be specified as part of the `Cargo.toml` definition.
当在项目中使用此 crate 作为依赖项时，微控制器可以 指定为定义的一部分.

```toml
[dependencies.stm32f1xx-hal]
version = "0.9.0"
features = ["stm32f100", "rt"]
```

## Documentation

The documentation can be found at [docs.rs](https://docs.rs/stm32f1xx-hal/).
该文档可在 docs.rs 中找到。

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
除非您另有明确说明，否则任何有意提交的贡献 为了包含在您的作品中，如 Apache-2.0 许可证中所定义，应为 双重许可，无任何其他附加条款或条件。