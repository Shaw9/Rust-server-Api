use napi_derive::napi;
use windows::Win32::UI::Shell::{Shell_NotifyIconW, NIIF_INFO, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW};
use windows::Win32::UI::WindowsAndMessaging::{GetDesktopWindow, LoadIconW, IDI_INFORMATION};

/// Windows 通知选项
#[napi(object)]
pub struct NotificationOptions {
  pub title: String,
  pub message: String,
}

/// Windows 通知结果
#[napi(object)]
pub struct NotificationResult {
  pub success: bool,
  pub message: String,
}

/// 发送 Windows 系统通知
#[napi]
#[cfg(windows)]
pub fn send_notification(options: NotificationOptions) -> NotificationResult {
  use std::iter;
  use std::ptr;
  use std::thread;

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

  // 创建 NOTIFYICONDATAW 结构体
  let mut nid = NOTIFYICONDATAW::default();
  nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;

  unsafe {
    // 获取桌面窗口句柄
    nid.hWnd = GetDesktopWindow();

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
    nid.dwInfoFlags = NIIF_INFO;
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
