#![deny(clippy::all)]
#![cfg(windows)]

mod windows;

pub use windows::{
  send_notification,
  NotificationOptions,
  NotificationResult,
  get_windows_info,
};
