pub mod send_notification;

pub use send_notification::{send_notification, NotificationOptions, NotificationResult};
pub mod get_windows_info;
pub use get_windows_info::{get_windows_info, WindowsInfo};