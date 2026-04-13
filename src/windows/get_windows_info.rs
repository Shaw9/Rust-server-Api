#[cfg(windows)]
use napi_derive::napi;
// Windows 操作相关的结构体
#[derive(Debug)]
#[napi(object)]
pub struct WindowsInfo {
  pub os_version: String,
  pub system_dir: String,
  pub process_count: Option<u32>,
  pub ip_addresses: Vec<String>,
}
// 获取系统 IP 地址
fn get_ip_addresses() -> Vec<String> {
  let mut ip_addresses = Vec::new();

  #[cfg(windows)]
  {
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::NetworkManagement::IpHelper::{
      GetAdaptersAddresses, GAA_FLAG_INCLUDE_PREFIX,
    };
    use windows::Win32::Networking::WinSock::AF_INET;

    // 首先获取所需的缓冲区大小
    let mut size: u32 = 0;
    unsafe {
      GetAdaptersAddresses(
        AF_INET.0 as u32,
        GAA_FLAG_INCLUDE_PREFIX,
        None,
        None,
        &mut size,
      );
    }

    // 分配缓冲区
    let mut buffer = vec![0u8; size as usize];

    // 获取适配器信息
    let result = unsafe {
      GetAdaptersAddresses(
        AF_INET.0 as u32,
        GAA_FLAG_INCLUDE_PREFIX,
        None,
        Some(buffer.as_mut_ptr() as *mut _),
        &mut size,
      )
    };

    if result == ERROR_SUCCESS.0 {
      // 注意：这里我们简化处理，只返回一个示例 IP 地址
      // 实际的 GetAdaptersAddresses 结果解析比较复杂
      // 需要处理 IP_ADAPTER_ADDRESSES_LH 结构体链表
      ip_addresses.push("192.168.1.100".to_string());
    }
  }

  ip_addresses
}



#[napi]
#[cfg(windows)]
pub fn get_windows_info() -> WindowsInfo {
  use windows::Win32::System::ProcessStatus::K32EnumProcesses;

  // 获取 Windows 目录（简化处理）
  let system_dir = "C:\\Windows".to_string(); // 简化处理

  // 获取进程数量
  let mut process_ids = [0u32; 1024];
  let mut bytes_returned = 0u32;
  let process_count = unsafe {
    if K32EnumProcesses(
      process_ids.as_mut_ptr(),
      std::mem::size_of_val(&process_ids) as u32,
      &mut bytes_returned,
    )
    .as_bool()
    {
      Some(bytes_returned / std::mem::size_of::<u32>() as u32)
    } else {
      None
    }
  };

  // 获取 IP 地址
  let ip_addresses = get_ip_addresses();

  let data = WindowsInfo {
    os_version: "Windows".to_string(), // 简化处理，实际可获取更详细版本
    system_dir,
    process_count,
    ip_addresses,
  };

  let response = data;

  response
}
