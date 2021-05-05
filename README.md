thingy52-lorawan-nbiot
===

Provides a pure Rust program for deployment to the nRF Thingy:52 device, which is an nRF9160
and a few sensors.

UDP is then used to convey LoRaWAN packets over a UDP connection.

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
cargo size --release
```

Flashing
---

> Note that this program is assuming the Thingy:91 device as a target and expects to find an environmental sensor.
> Assuming an SWD cable is being used, it is useful to first flash "blinky" to the SWD host device e.g. the
> nRF9160-DK board. You then have assurance that what is being flashed to the Thingy:91 (unless that too is Blinky!)
> is your software. These DK boards provide no feedback that a debug session is in place! For convenience, there's a
> blinky app provided with this project, which just runs in secure mode and so doesn't require an additional SPM as
> described below).

By default, the program is flashed to memory address 0x0005_0000. A Nordic Secure Partition Manager program is also required
to be flashed to the device. [Nordic provide one](https://github.com/nrfconnect/sdk-nrf/tree/master/samples/spm) which jumps
to this address. A built version of it exists in the root folder and is named "spm.hex". This SPM needs to be flashed to the 
device before flashing/debugging the main program here e.g.:

```
nrfjprog --program ./spm.hex --sectorerase
```

Structure
---

The project has an `applib` sub project to hold general app logic that can be tested off the board.
This project could also well be factored out into other crates if the need arises. The `app` project
specifically targets the Thingy:91 device. Note that you need to be within the `app` project
to build it i.e. it isn't able to be part of the workspace given its target.