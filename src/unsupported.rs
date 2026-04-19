use napi_derive::napi;

#[napi(object)]
pub struct NotificationOptions {
  pub title: String,
  pub message: String,
}

#[napi(object)]
pub struct NotificationResult {
  pub success: bool,
  pub message: String,
}

#[napi(object)]
pub struct WindowsInfo {
  pub os_version: String,
  pub system_dir: String,
  pub process_count: Option<u32>,
  pub ip_addresses: Vec<String>,
}

#[napi]
pub fn send_notification(_options: NotificationOptions) -> NotificationResult {
  NotificationResult {
    success: false,
    message: format!(
      "Windows notifications are not supported on {}",
      std::env::consts::OS,
    ),
  }
}

#[napi]
pub fn get_windows_info() -> WindowsInfo {
  WindowsInfo {
    os_version: std::env::consts::OS.to_string(),
    system_dir: String::new(),
    process_count: None,
    ip_addresses: Vec::new(),
  }
}
