# Rust SocketCAN

This library implements Controller Area Network (CAN) communications on Linux using the SocketCAN subsystem. This provides a network socket interface to the CAN bus.

[Linux SocketCAN](https://docs.kernel.org/networking/can.html)

Please see the [documentation](https://docs.rs/socketcan) for details about the Rust API provided by this library.


## Latest News

Then end of 2024 saw two back-to-back release, v3.4 and v3.5. The first rolled up the PRs and bug fixes that had been sitting the the repository for the year. The second updated the `dump` module and improved the usability of several frame and socket types. See below for details.

### Version 3.x adds integrated async/await, improved Netlink coverage, and more!

Version 3.0 adds integrated support for async/await, with the most popular runtimes, _tokio, async-std_, and _smol_.  We have merged the [tokio-socketcan](https://github.com/oefd/tokio-socketcan) crate into this one and implemented `async-io`.

Unfortunately this required a minor breaking change to the existing API, so we bumped the version to 3.0.

The async support is optional, and can be enabled with a feature for the target runtime: `tokio`, `async-std`, or `smol`.

Additional implementation of the netlink control of the CAN interface was added in v3.1 allowing an application to do things like set the bitrate on the interface, set control modes, restart the interface, etc.

v3.2 increased the interface configuration coverage with Netlink, allowing an application to set most interface CAN parameters and query them all back.

### What's New in Version 3.5

- `CanAnyFrame` implements `From` trait for `CanDataFrame`, `CanRemoteFrame`, and `CanErrorFrame`.
- `CanFdSocket` implementa `TryFrom` trait for `CanSocket`
- Added FdFlags::FDF bit mask for CANFD_FDF
    - The FDF flag is forced on when creating a CanFdFrame.
- Updates to `dump` module:
    - Re-implemented with text parsing
    - `ParseError` now implements std `Error` trait via `thiserror::Error` 
    - Parses FdFlags field properly 
    - CANFD_FDF bit flag recognized on input
    - Fixed reading remote frames
    - Now reads remote length
    - `CanDumpRecord` changes:
	- Removed lifetime and made `device` field an owned `String`
	- Implemented `Clone` and `Display` traits.
	- `Display` trait is compatible with the candump log record format
    - `dump::Reader` is now an Iterator itself, returning full `CanDumpRecord` items
    - New unit tests
- [#59](https://github.com/socketcan-rs/socketcan-rs/issues/59) Embedded Hal for CanFdSocket

### What's New in Version 3.4

Version 3.4.0 was primarily a service release to publish a number of new feature and bug fixes that had accumulated in the repository over the previous months. Those included:

- A new build feature, `enumerate` to provide code for enumerating CAN interfaces on the host.
- Added a `CanId` type with better usability than `embedded_can::Id`
- Fixes to the Flexible Data (CAN FD) implementation, including proper frame padding and DLC/length queries.
- Made `CanState` public.
- Improved and modernized the _tokio_ implementation.

For a full list of updates, see the [v3.4.0 CHANGELOG](https://github.com/socketcan-rs/socketcan-rs/blob/v3.4.0/CHANGELOG.md)

## Next Steps

A number of items still did not make it into a release. These will be added in v3.x, coming soon.

- Issue [#22](https://github.com/socketcan-rs/socketcan-rs/issues/22) Timestamps, including optional hardware timestamps

## Minimum Supported Rust Version (MSRV)

The current version of the crate targets Rust Edition 2021 with an MSRV of Rust v1.70.

Note that, the core library can likely compile with an earlier version if dependencies are carefully selected, but tests are being done with the latest stable compiler and the stated MSRV.

## Async Support

### Tokio

The [tokio-socketcan](https://crates.io/crates/tokio-socketcan) crate was merged into this one to provide async support for CANbus using tokio.

This is enabled with the optional feature, `tokio`.

#### Example bridge with _tokio_

This is a simple example of sending data frames from one CAN interface to another. It is included in
the example applications as
[tokio_bridge.rs](https://github.com/socketcan-rs/socketcan-rs/blob/master/examples/tokio_bridge.rs).

```rust
use futures_util::StreamExt;
use socketcan::{tokio::CanSocket, CanFrame, Result};
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let mut sock_rx = CanSocket::open("vcan0")?;
    let sock_tx = CanSocket::open("can0")?;

    while let Some(Ok(frame)) = sock_rx.next().await {
        if matches!(frame, CanFrame::Data(_)) {
            sock_tx.write_frame(frame)?.await?;
        }
    }

    Ok(())
}
```

### async-io  (_async-std_ & _smol_)

New support was added for the [async-io](https://crates.io/crates/async-io) runtime, supporting the [async-std](https://crates.io/crates/async-std) and [smol](https://crates.io/crates/smol) runtimes.

This is enabled with the optional feature, `async-io`. It can also be enabled with either feature, `async-std` or `smol`. Either of those specific runtime flags will simply build the `async-io` support but then also alias the `async-io` submodule with the specific feature/runtime name. This is simply for convenience.

Additionally, when building examples, the specific examples for the runtime will be built if specifying the `async-std` or `smol` feature(s).

#### Example bridge with _async-std_

This is a simple example of sending data frames from one CAN interface to another. It is included in
the example applications as
[async_std_bridge.rs](https://github.com/socketcan-rs/socketcan-rs/blob/master/examples/async_std_bridge.rs).

```rust
use socketcan::{async_std::CanSocket, CanFrame, Result};

#[async_std::main]
async fn main() -> Result<()> {
    let sock_rx = CanSocket::open("vcan0")?;
    let sock_tx = CanSocket::open("can0")?;

    loop {
        let frame = sock_rx.read_frame().await?;
        if matches!(frame, CanFrame::Data(_)) {
            sock_tx.write_frame(&frame).await?;
        }
    }
}
```

## Testing

Integrating the full suite of tests into a CI system is non-trivial as it relies on a `vcan0` virtual CAN device existing. Adding it to most Linux systems is pretty easy with root access, but attaching a vcan device to a container for CI seems difficult to implement.

Therefore, tests requiring `vcan0` were placed behind an optional feature, `vcan_tests`.

The steps to install and add a virtual interface to Linux are in the `scripts/vcan.sh` script. Run it with root proveleges, then run the tests:

```sh
$ sudo ./scripts/vcan.sh
$ cargo test --features=vcan_tests
```