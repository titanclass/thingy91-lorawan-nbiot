thingy91-lorawan-nbiot
===

Provides a pure Rust program for deployment to the nRF Thingy:91 device, which is an nRF9160
and a few sensors.

UDP is then used to convey LoRaWAN packets over a UDP connection on a periodic basis.

Development
---

You'll need the following toolchain:

```
rustup target add thumbv8m.main-none-eabi
```

...along with the Arm/gcc toolchain: https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads.

To see the size of a resulting binary,
`cargo install cargo-binutils` and `rustup component add llvm-tools-preview` will install tools to permit you to determine the size of a release binary e.g.:

```
cd app
cargo size --release --target thumbv8m.main-none-eabihf
```

Flashing the SPM
---

> Note that this program is assuming the Thingy:91 device as a target and expects to find an environmental sensor.
> Assuming an SWD cable is being used, it is useful to first flash "blinky" to the SWD host device e.g. the
> nRF9160-DK board. You then have assurance that what is being flashed to the Thingy:91 (unless that too is Blinky!)
> is your software. These DK boards provide no feedback that a debug session is in place! For convenience, there's a
> blinky app provided with this project, which just runs in secure mode and so doesn't require an additional SPM as
> described below).

By default, the program is flashed to memory address 0x0004_0000. A Nordic Secure Partition Manager program is also required
to be flashed to the device. [Nordic provide one](https://github.com/nrfconnect/sdk-nrf/tree/master/samples/spm) which jumps
to this address. This project provides an `spm` for convenience. However, to build the spm is (from your Nordic/Zephyr installation):

```
source zephyr/zephyr-env.sh
west build -b thingy91_nrf9160 nrf/samples/spm --pristine
west flash
```

Testing the app
---

```
cargo test
```

Running the app
---

```
cargo run --target thumbv8m.main-none-eabihf
```

Configuring the app
---

This application uses a special mode so that a user may configure it. We enter this special mode to 
conserve battery by shutting down the UART when in regular running mode.

To configure the app, you need to connect to the Thingy:91's serial port. You will also need to configure your
serial port to add carriage returns to line feeds. `minicom` is a utility that easily enables this e.g. if
the serial port is `/dev/tty.usbmodem143101`:

```
minicom -D /dev/tty.usbmodem143101
```

When connected to the serial port, hold down the Thingy:91's button during start up. This will place you into 
"command mode". At any time, type "help" to see what can be done. Resetting the device exits command mode. Pressing
the escape also causes the device to exit command mode and reset.

Structure
---

The project has an `app` sub project to hold general app logic that can be tested off the board.
This project could also well be factored out into other crates if the need arises. The `nrf-app` project
specifically targets the Thingy:91 device. Note that you need to be within the `nrf-app` project
to build it i.e. it isn't able to be part of the workspace given its target.

## Contribution policy

Contributions via GitHub pull requests are gladly accepted from their original author. Along with any pull requests, please state that the contribution is your original work and that you license the work to the project under the project's open source license. Whether or not you state this explicitly, by submitting any copyrighted material via pull request, email, or other means you agree to license the material under the project's open source license and warrant that you have the legal authority to do so.

## License

This code is open source software licensed under the [Apache-2.0 license](./LICENSE).

© Copyright [Titan Class P/L](https://www.titanclass.com.au/), 2021
