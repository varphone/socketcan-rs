// socketcan/src/lib.rs
//
// The main lib file for the Rust SocketCAN library.
//
// This file is part of the Rust 'socketcan-rs' library.
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.

//! SocketCAN support.
//!
//! The Linux kernel supports using CAN-devices through a network-like API
//! (see <https://www.kernel.org/doc/Documentation/networking/can.txt>). This
//! crate allows easy access to this functionality without having to wrestle
//! libc calls.
//!
//! # An introduction to CAN
//!
//! The CAN bus was originally designed to allow microcontrollers inside a
//! vehicle to communicate over a single shared bus. Messages called
//! *frames* are multicast to all devices on the bus.
//!
//! Every frame consists of an ID and a payload of up to 8 bytes. If two
//! devices attempt to send a frame at the same time, the device with the
//! higher ID will notice the conflict, stop sending and reattempt to sent its
//! frame in the next time slot. This means that the lower the ID, the higher
//! the priority. Since most devices have a limited buffer for outgoing frames,
//! a single device with a high priority (== low ID) can block communication
//! on that bus by sending messages too fast.
//!
//! The CAN Flexible Data-Rate (CAN FD) standard extended the data payload up to
//! 64 bytes and added the ability to increase the the bitrate for the data bit
//! in the frame.
//!
//! The Linux socketcan subsystem makes the CAN bus available as a regular
//! networking device. Opening a network interface allows an application to
//! receive all CAN messages from the bus and/or to filter for specific messages
//! based on the CAN ID field. A device can be opened multiple times, every
//! client will receive all CAN frames simultaneously.
//!
//! Similarly, CAN frames can be sent to the bus by multiple client
//! simultaneously as well.
//!
//! # Hardware and more information
//!
//! More information on CAN can be found on
//! [Wikipedia](https://en.wikipedia.org/wiki/CAN_bus).
//! When not running on an embedded platform with already integrated CAN components,
//! [Thomas Fischl's USBtin](http://www.fischl.de/usbtin/) (see
//! [section 2.4](http://www.fischl.de/usbtin/#socketcan)) is one of many ways
//! to get started.
//!
//! # RawFd and OwnedFd
//!
//! Raw access to the underlying file descriptor and construction through one
//! is available through the `AsRawFd`, `IntoRawFd` and `FromRawFd`, and
//! similar implementations.
//!
//! # Crate Features
//!
//! ### Default
//!
//! * **netlink** -
//!   Whether to include programmable CAN interface configuration capabilities
//!   based on netlink kernel communications. This brings in the
//!   [neli](https://docs.rs/neli/latest/neli/) library and its dependencies.
//!
//! * **dump** -
//!   Whether to include candump parsing capabilities.
//!
//! ### Non-default
//!
//! * **enumerate** -
//!   Include the `enumerate` module which can be used to get a list of the CANbus
//!   network interfaces attached to the host. This brings in the dependency for
//!   [libudev](https://crates.io/crates/libudev)
//!
//! * **utils** -
//!   Whether to build command-line utilities. This brings in additional
//!   dependencies like [anyhow](https://docs.rs/anyhow/latest/anyhow/) and
//!   [clap](https://docs.rs/clap/latest/clap/)
//!
//! * **tokio** -
//!   Include support for async/await using [tokio](https://crates.io/crates/tokio).
//!
//! * **async-io** -
//!   Include support for async/await using [async-io](https://crates.io/crates/async-io)
//!   This will work with any runtime that uses _async_io_, including
//!   [async-std](https://crates.io/crates/async-std) and [smol](https://crates.io/crates/smol).
//!
//! * **async-std** -
//!   Include support for async/await using [async-io](https://crates.io/crates/async-io)
//!   with a submodule aliased for [async-std](https://crates.io/crates/async-std) and examples
//!   for that runtime.
//!
//! * **smol** -
//!   Include support for async/await using [async-io](https://crates.io/crates/async-io)
//!   with a submodule aliased for [smol](https://crates.io/crates/smol) and examples
//!   for that runtime.
//!
//! ### Test Features
//!
//! Additional test can be built and run, but have requirements:
//!
//! * **vcan_tests** -
//!   Requires a virtual CAN interface to be installed on the host. This can be done
//!   by running the `vcan.sh` script included with the crate.
//!
//! * **netlink_tests** -
//!   Requires superuser privileges to run/pass.
//!

// clippy: do not warn about things like "SocketCAN" inside the docs
#![allow(clippy::doc_markdown)]
// Some lints
#![deny(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    unsafe_op_in_unsafe_fn
)]

use std::mem::size_of;

// Re-export the embedded_can crate so that applications can rely on
// finding the same version we use.
pub use embedded_can::{
    self, blocking::Can as BlockingCan, nb::Can as NonBlockingCan, ExtendedId,
    Frame as EmbeddedFrame, Id, StandardId,
};

pub mod errors;
pub use errors::{
    CanError, CanErrorDecodingFailure, ConstructionError, Error, IoError, IoErrorKind, IoResult,
    Result,
};

pub mod addr;
pub use addr::CanAddr;

pub mod id;
pub use id::CanId;

pub mod frame;
pub use frame::{
    CanAnyFrame, CanDataFrame, CanErrorFrame, CanFdFrame, CanFrame, CanRawFrame, CanRemoteFrame,
    Frame,
};

#[cfg(feature = "dump")]
pub mod dump;

pub mod socket;
pub use socket::{CanFdSocket, CanFilter, CanSocket, ShouldRetry, Socket, SocketOptions};

#[cfg(feature = "netlink")]
pub mod nl;

#[cfg(feature = "netlink")]
pub use nl::{CanCtrlMode, CanInterface, InterfaceCanParams};

/// Optional tokio support
#[cfg(feature = "tokio")]
pub mod tokio;

/// Optional support for async-io-based async runtimes, like async-std and smol.
#[cfg(any(feature = "async-io", feature = "async-std", feature = "smol"))]
pub mod async_io;

/// Using the specific definition for 'smol', just re-export the async_io module.
#[cfg(feature = "smol")]
pub mod smol {
    pub use crate::async_io::*;
}

/// Using the specific definition for 'async_std', just re-export the async_io module.
#[cfg(feature = "async-std")]
pub mod async_std {
    pub use crate::async_io::*;
}

#[cfg(feature = "enumerate")]
pub mod enumerate;
#[cfg(feature = "enumerate")]
pub use enumerate::available_interfaces;

// ===== helper functions =====

/// Gets a byte slice for any sized variable.
///
/// Note that this should normally be unsafe, but since we're only
/// using it internally for types sent to the kernel, it's OK.
pub(crate) fn as_bytes<T: Sized>(val: &T) -> &[u8] {
    let sz = size_of::<T>();
    unsafe { std::slice::from_raw_parts::<'_, u8>(val as *const _ as *const u8, sz) }
}

/// Gets a mutable byte slice for any sized variable.
pub(crate) fn as_bytes_mut<T: Sized>(val: &mut T) -> &mut [u8] {
    let sz = size_of::<T>();
    unsafe { std::slice::from_raw_parts_mut(val as *mut _ as *mut u8, sz) }
}
