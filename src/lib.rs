#![deny(clippy::all)]
#![cfg(windows)]

use napi_derive::napi;

mod windows;

pub use windows::{
  send_notification,
  NotificationOptions,
  NotificationResult,
};

#[napi]
pub fn add(a: u32, b: u32) -> u32 {
  a + b
}
