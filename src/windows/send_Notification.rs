use std::path::PathBuf;
use std::sync::OnceLock;

use napi_derive::napi;
use windows::{
  Data::Xml::Dom::XmlDocument,
  UI::Notifications::{ToastNotification, ToastNotificationManager},
  Win32::{
    Foundation::{RPC_E_CHANGED_MODE, S_FALSE},
    Storage::EnhancedStorage::{PKEY_AppUserModel_ID, PKEY_AppUserModel_ToastActivatorCLSID},
    System::{
      Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_MULTITHREADED, IPersistFile,
      },
      WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED},
    },
    UI::Shell::{
      PropertiesSystem::IPropertyStore, FOLDERID_Programs, KF_FLAG_CREATE, SHGetKnownFolderPath,
      IShellLinkW, ShellLink,
    },
  },
  core::{Error as WinError, GUID, HSTRING, Interface},
};

const APP_ID: &str = "ShawLiu.server_rust_api";
const SHORTCUT_NAME: &str = "server_rust_api.lnk";
const SHORTCUT_DESCRIPTION: &str = "server_rust_api notifications";
const TOAST_ACTIVATOR_CLSID: GUID = GUID::from_u128(0xb7f0a0c8_5f5d_4f5b_9018_6d2424c2b247);

static SHORTCUT_READY: OnceLock<Result<(), String>> = OnceLock::new();

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

#[derive(Default)]
struct RuntimeInitGuard {
  co_uninitialize: bool,
  ro_uninitialize: bool,
}

impl Drop for RuntimeInitGuard {
  fn drop(&mut self) {
    if self.ro_uninitialize {
      unsafe {
        RoUninitialize();
      }
    }

    if self.co_uninitialize {
      unsafe {
        CoUninitialize();
      }
    }
  }
}

/// 发送 Windows 系统通知
#[napi]
#[cfg(windows)]
pub fn send_notification(options: NotificationOptions) -> NotificationResult {
  match send_toast_notification(options) {
    Ok(()) => NotificationResult {
      success: true,
      message: "通知发送成功".to_string(),
    },
    Err(error) => NotificationResult {
      success: false,
      message: error,
    },
  }
}

fn send_toast_notification(options: NotificationOptions) -> Result<(), String> {
  let _runtime = initialize_runtime()?;
  ensure_shortcut_registered()?;

  let xml = build_toast_xml(&options);
  let document = XmlDocument::new().map_err(|error| format_windows_error("创建通知 XML 失败", &error))?;
  document
    .LoadXml(&HSTRING::from(xml))
    .map_err(|error| format_windows_error("加载通知 XML 失败", &error))?;

  let toast = ToastNotification::CreateToastNotification(&document)
    .map_err(|error| format_windows_error("创建 ToastNotification 失败", &error))?;
  let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(APP_ID))
    .map_err(|error| format_windows_error("创建通知发送器失败", &error))?;

  notifier
    .Show(&toast)
    .map_err(|error| format_windows_error("发送通知失败", &error))?;

  Ok(())
}

fn initialize_runtime() -> Result<RuntimeInitGuard, String> {
  let mut guard = RuntimeInitGuard::default();

  let co_init = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
  if co_init == RPC_E_CHANGED_MODE {
    // 当前线程已由宿主以其他模式初始化，继续复用现有 COM 上下文。
  } else if co_init.is_ok() {
    guard.co_uninitialize = true;
  } else {
    return Err(format_hresult("初始化 COM 失败", co_init));
  }

  match unsafe { RoInitialize(RO_INIT_MULTITHREADED) } {
    Ok(()) => {
      guard.ro_uninitialize = true;
    }
    Err(error) if error.code() == RPC_E_CHANGED_MODE => {
      // WinRT 已由宿主初始化，继续使用即可。
    }
    Err(error) => {
      return Err(format_windows_error("初始化 WinRT 失败", &error));
    }
  }

  Ok(guard)
}

fn ensure_shortcut_registered() -> Result<(), String> {
  SHORTCUT_READY
    .get_or_init(register_shortcut)
    .as_ref()
    .map(|_| ())
    .map_err(Clone::clone)
}

fn register_shortcut() -> Result<(), String> {
  let shortcut_path = shortcut_path()?;
  let executable_path = std::env::current_exe().map_err(|error| format!("获取当前进程路径失败: {error}"))?;
  let working_directory = executable_path
    .parent()
    .ok_or_else(|| "当前进程没有可用的工作目录".to_string())?;

  let shell_link: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }
    .map_err(|error| format_windows_error("创建 ShellLink 失败", &error))?;

  let executable = HSTRING::from(executable_path.to_string_lossy().as_ref());
  let working_directory = HSTRING::from(working_directory.to_string_lossy().as_ref());
  let shortcut_path_value = HSTRING::from(shortcut_path.to_string_lossy().as_ref());
  let description = HSTRING::from(SHORTCUT_DESCRIPTION);

  unsafe {
    shell_link
      .SetPath(&executable)
      .map_err(|error| format_windows_error("设置快捷方式目标失败", &error))?;
    shell_link
      .SetWorkingDirectory(&working_directory)
      .map_err(|error| format_windows_error("设置快捷方式工作目录失败", &error))?;
    shell_link
      .SetDescription(&description)
      .map_err(|error| format_windows_error("设置快捷方式描述失败", &error))?;
    shell_link
      .SetIconLocation(&executable, 0)
      .map_err(|error| format_windows_error("设置快捷方式图标失败", &error))?;
  }

  let property_store: IPropertyStore = shell_link
    .cast()
    .map_err(|error| format_windows_error("获取快捷方式属性存储失败", &error))?;
  let app_id_value = windows::Win32::System::Com::StructuredStorage::PROPVARIANT::from(APP_ID);
  let toast_activator_value = unsafe { windows::Win32::System::Com::StructuredStorage::InitPropVariantFromCLSID(&TOAST_ACTIVATOR_CLSID) }
    .map_err(|error| format_windows_error("创建 ToastActivatorCLSID 属性失败", &error))?;

  unsafe {
    property_store
      .SetValue(&PKEY_AppUserModel_ID, &app_id_value)
      .map_err(|error| format_windows_error("设置 AppUserModelID 失败", &error))?;
    property_store
      .SetValue(&PKEY_AppUserModel_ToastActivatorCLSID, &toast_activator_value)
      .map_err(|error| format_windows_error("设置 ToastActivatorCLSID 失败", &error))?;
    property_store
      .Commit()
      .map_err(|error| format_windows_error("提交快捷方式属性失败", &error))?;
  }

  let persist_file: IPersistFile = shell_link
    .cast()
    .map_err(|error| format_windows_error("获取快捷方式保存接口失败", &error))?;
  unsafe {
    persist_file
      .Save(&shortcut_path_value, true)
      .map_err(|error| format_windows_error("保存开始菜单快捷方式失败", &error))?;
  }
  Ok(())
}

fn shortcut_path() -> Result<PathBuf, String> {
  let raw_path = unsafe { SHGetKnownFolderPath(&FOLDERID_Programs, KF_FLAG_CREATE, None) }
    .map_err(|error| format_windows_error("获取开始菜单路径失败", &error))?;

  let programs_path = unsafe {
    let result = raw_path
      .to_string()
      .map(PathBuf::from)
      .map_err(|error| format!("解析开始菜单路径失败: {error}"));
    CoTaskMemFree(Some(raw_path.0 as _));
    result
  }?;

  Ok(programs_path.join(SHORTCUT_NAME))
}

fn build_toast_xml(options: &NotificationOptions) -> String {
  let title = escape_xml(&options.title);
  let message = escape_xml(&options.message);

  format!(
    r#"<toast><visual><binding template="ToastGeneric"><text>{title}</text><text>{message}</text></binding></visual></toast>"#,
  )
}

fn escape_xml(value: &str) -> String {
  value
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&apos;")
}

fn format_windows_error(context: &str, error: &WinError) -> String {
  format!("{context}: {error}")
}

fn format_hresult(context: &str, hr: windows::core::HRESULT) -> String {
  if hr == S_FALSE {
    return context.to_string();
  }

  let error = WinError::from(hr);
  format_windows_error(context, &error)
}
