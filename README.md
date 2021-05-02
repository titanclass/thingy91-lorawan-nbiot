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

Structure
---

The project has an `applib` sub project to hold general app logic that can be tested off the board.
This project could also well be factored out into other crates if the need arises. The `app` project
specifically targets the Thingy:91 device. Note that you need to be within the `app` project
to build it i.e. it isn't able to be part of the workspace given its target.