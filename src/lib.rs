#![deny(clippy::all)]
#![cfg(windows)]

use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use napi_derive::napi;
use windows::Win32::UI::Shell::NIIF_USER;
use windows::Win32::UI::WindowsAndMessaging::{CreateIcon, DestroyIcon, HICON};

#[napi]
pub fn add(a: u32, b: u32) -> u32 {
  a + b
}

/// Windows 通知选项
#[napi(object)]
pub struct NotificationOptions {
  pub title: String,
  pub message: String,
  pub icon: Option<String>,
}

/// Windows 通知结果
#[napi(object)]
pub struct NotificationResult {
  pub success: bool,
  pub message: String,
}

struct OwnedIcon(HICON);

impl Drop for OwnedIcon {
  fn drop(&mut self) {
    unsafe {
      let _ = DestroyIcon(self.0);
    }
  }
}

fn decode_icon_base64(input: &str) -> Result<Vec<u8>, String> {
  let payload = input
    .trim()
    .split_once(',')
    .map(|(_, data)| data)
    .unwrap_or(input)
    .chars()
    .filter(|char| !char.is_whitespace())
    .collect::<String>();

  for engine in [&STANDARD, &STANDARD_NO_PAD, &URL_SAFE, &URL_SAFE_NO_PAD] {
    if let Ok(decoded) = engine.decode(&payload) {
      return Ok(decoded);
    }
  }

  Err("通知图标不是有效的 base64 数据".to_string())
}

fn create_icon_from_base64(icon_base64: &str) -> Result<OwnedIcon, String> {
  let image_bytes = decode_icon_base64(icon_base64)?;
  let image = image::load_from_memory(&image_bytes)
    .map_err(|error| format!("解析通知图标失败: {error}"))?
    .into_rgba8();
  let (width, height) = image.dimensions();

  if width == 0 || height == 0 {
    return Err("通知图标不能为空图片".to_string());
  }

  if width > i32::MAX as u32 || height > i32::MAX as u32 {
    return Err("通知图标尺寸过大".to_string());
  }

  let mut bgra = image.into_raw();
  let pixel_count = bgra.len() / 4;
  let mut and_mask = Vec::with_capacity(pixel_count);

  for pixel in bgra.chunks_exact_mut(4) {
    and_mask.push(pixel[3].wrapping_sub(u8::MAX));
    pixel.swap(0, 2);
  }

  unsafe {
    CreateIcon(
      None,
      width as i32,
      height as i32,
      1,
      32,
      and_mask.as_ptr(),
      bgra.as_ptr(),
    )
    .map(OwnedIcon)
    .map_err(|error| format!("创建通知图标失败: {error}"))
  }
}

/// 发送 Windows 系统通知
#[napi]
pub fn send_notification(options: NotificationOptions) -> NotificationResult {
  use std::iter;
  use std::ptr;
  use std::thread;
  use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIIF_INFO, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
  };
  use windows::Win32::UI::WindowsAndMessaging::{GetDesktopWindow, LoadIconW, IDI_INFORMATION};

  // 定义常量
  const NIF_INFO: u32 = 0x00000010;
  const NIF_ICON: u32 = 0x00000002;
  const NIF_TIP: u32 = 0x00000004;

  // 转换为宽字符串
  let title_wide: Vec<u16> = options.title.encode_utf16().chain(iter::once(0)).collect();
  let message_wide: Vec<u16> = options
    .message
    .encode_utf16()
    .chain(iter::once(0))
    .collect();
  let custom_icon = match options.icon.as_deref().map(str::trim) {
    Some(icon) if !icon.is_empty() => match create_icon_from_base64(icon) {
      Ok(icon) => Some(icon),
      Err(message) => {
        return NotificationResult {
          success: false,
          message,
        };
      }
    },
    _ => None,
  };

  // 创建 NOTIFYICONDATAW 结构体
  let mut nid = NOTIFYICONDATAW::default();
  nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;

  unsafe {
    // 获取桌面窗口句柄
    nid.hWnd = GetDesktopWindow();

    match custom_icon.as_ref() {
      Some(icon) => {
        nid.hIcon = icon.0;
        nid.hBalloonIcon = icon.0;
      }
      None => {
        // 加载默认信息图标
        match LoadIconW(None, IDI_INFORMATION) {
          Ok(icon) => nid.hIcon = icon,
          Err(_) => {
            return NotificationResult {
              success: false,
              message: "加载默认图标失败".to_string(),
            };
          }
        }
      }
    }

    // 设置通知图标 ID
    nid.uID = 1;

    // 设置标志
    nid.uFlags = windows::Win32::UI::Shell::NOTIFY_ICON_DATA_FLAGS(NIF_INFO | NIF_ICON | NIF_TIP);

    // 设置气球通知标题
    ptr::copy_nonoverlapping(
      title_wide.as_ptr(),
      nid.szInfoTitle.as_mut_ptr(),
      std::cmp::min(title_wide.len(), nid.szInfoTitle.len() - 1),
    );

    // 设置气球通知内容
    ptr::copy_nonoverlapping(
      message_wide.as_ptr(),
      nid.szInfo.as_mut_ptr(),
      std::cmp::min(message_wide.len(), nid.szInfo.len() - 1),
    );

    // 设置通知图标类型
    nid.dwInfoFlags = if custom_icon.is_some() {
      NIIF_USER
    } else {
      NIIF_INFO
    };
  }

  // 显示通知
  let result = unsafe { Shell_NotifyIconW(NIM_ADD, &nid) };
  let success = result.as_bool();

  if success {
    // 延迟删除通知图标（使用同步睡眠，不跨线程）
    thread::sleep(std::time::Duration::from_secs(5));
    unsafe {
      let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
    }
  }

  NotificationResult {
    success,
    message: if success {
      "通知发送成功".to_string()
    } else {
      "通知发送失败".to_string()
    },
  }
}

#[cfg(test)]
mod tests {
  use super::decode_icon_base64;

  #[test]
  fn decode_plain_base64_icon_payload() {
    let decoded = decode_icon_base64("SGVsbG8=").expect("plain base64 should decode");
    assert_eq!(decoded, b"Hello");
  }

  #[test]
  fn decode_data_url_icon_payload() {
    let decoded =
      decode_icon_base64("data:image/png;base64,SGVsbG8=").expect("data url should decode");
    assert_eq!(decoded, b"Hello");
  }
}
