#![deny(clippy::all)]

#[cfg(not(windows))]
mod unsupported;
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
pub use unsupported::{
  get_windows_info, send_notification, NotificationOptions, NotificationResult, WindowsInfo,
};
#[cfg(windows)]
pub use windows::{
  get_windows_info, send_notification, NotificationOptions, NotificationResult, WindowsInfo,
};
