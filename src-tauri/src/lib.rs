use std::process::Command;

#[cfg(target_os = "macos")]
use std::{ffi::c_void, thread, time::Duration};

fn split_shortcut(shortcut: &str) -> (Vec<String>, String) {
    let mut parts: Vec<String> = shortcut
        .split('+')
        .map(|part| part.trim().to_lowercase())
        .filter(|part| !part.is_empty())
        .collect();
    let key = parts.pop().unwrap_or_default();
    (parts, key)
}

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> u8;
}

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventCreateKeyboardEvent(
        source: *mut c_void,
        virtual_key: u16,
        key_down: bool,
    ) -> *mut c_void;
    fn CGEventSetFlags(event: *mut c_void, flags: u64);
    fn CGEventPost(tap: u32, event: *mut c_void);
}

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(value: *const c_void);
}

#[cfg(target_os = "macos")]
const MAC_FLAG_SHIFT: u64 = 0x0002_0000;
#[cfg(target_os = "macos")]
const MAC_FLAG_CONTROL: u64 = 0x0004_0000;
#[cfg(target_os = "macos")]
const MAC_FLAG_OPTION: u64 = 0x0008_0000;
#[cfg(target_os = "macos")]
const MAC_FLAG_COMMAND: u64 = 0x0010_0000;

#[cfg(target_os = "macos")]
fn mac_accessibility_allowed() -> bool {
    unsafe { AXIsProcessTrusted() != 0 }
}

#[cfg(target_os = "macos")]
fn mac_app_name(target_app: &str) -> String {
    let name = target_app.trim().trim_end_matches(".app").trim();
    if name.is_empty() {
        "Nuendo 15".into()
    } else {
        name.into()
    }
}

#[cfg(target_os = "macos")]
fn mac_is_running(target_app: &str) -> bool {
    let exact = Command::new("/usr/bin/pgrep")
        .args(["-x", target_app])
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    if exact {
        return true;
    }
    Command::new("/usr/bin/pgrep")
        .args(["-f", "/Nuendo [0-9]+\\.app/Contents/MacOS/Nuendo"])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[cfg(target_os = "macos")]
fn mac_focus(target_app: &str) -> Result<String, String> {
    let app_name = mac_app_name(target_app);
    if !mac_is_running(&app_name) {
        return Err(format!(
            "找不到 {app_name}。請先開啟 Nuendo，或在設定中修改 macOS Nuendo App 名稱。"
        ));
    }
    if !mac_accessibility_allowed() {
        return Err("Nuendo Conform Helper 尚未取得 macOS 輔助使用權限。請按「開啟輔助使用設定」，允許本 App 後完全結束並重新開啟。".into());
    }
    let status = Command::new("/usr/bin/open")
        .args(["-a", &app_name])
        .status()
        .map_err(|error| format!("無法切換到 {app_name}：{error}"))?;
    if !status.success() {
        return Err(format!("無法切換到 {app_name}。請確認 App 名稱設定正確。"));
    }
    Ok(app_name)
}

#[cfg(target_os = "macos")]
fn mac_key_code(key: &str) -> Option<u16> {
    Some(match key {
        "a" => 0,
        "s" => 1,
        "d" => 2,
        "f" => 3,
        "h" => 4,
        "g" => 5,
        "z" => 6,
        "x" => 7,
        "c" => 8,
        "v" => 9,
        "b" => 11,
        "q" => 12,
        "w" => 13,
        "e" => 14,
        "r" => 15,
        "y" => 16,
        "t" => 17,
        "1" => 18,
        "2" => 19,
        "3" => 20,
        "4" => 21,
        "6" => 22,
        "5" => 23,
        "=" => 24,
        "9" => 25,
        "7" => 26,
        "-" => 27,
        "8" => 28,
        "0" => 29,
        "]" => 30,
        "o" => 31,
        "u" => 32,
        "[" => 33,
        "i" => 34,
        "p" => 35,
        "enter" | "return" => 36,
        "l" => 37,
        "j" => 38,
        "'" => 39,
        "k" => 40,
        ";" => 41,
        "\\" => 42,
        "," => 43,
        "/" => 44,
        "n" => 45,
        "m" => 46,
        "." => 47,
        "tab" => 48,
        "space" => 49,
        "`" => 50,
        "backspace" => 51,
        "esc" | "escape" => 53,
        "f5" => 96,
        "f6" => 97,
        "f7" => 98,
        "f3" => 99,
        "f8" => 100,
        "f9" => 101,
        "f11" => 103,
        "f10" => 109,
        "f12" => 111,
        "home" => 115,
        "pageup" => 116,
        "delete" => 117,
        "f4" => 118,
        "end" => 119,
        "f2" => 120,
        "pagedown" => 121,
        "f1" => 122,
        "left" => 123,
        "right" => 124,
        "down" => 125,
        "up" => 126,
        _ => return None,
    })
}

#[cfg(target_os = "macos")]
fn mac_shortcut(shortcut: &str) -> Result<(u16, u64), String> {
    let (modifiers, key) = split_shortcut(shortcut);
    if key.is_empty() {
        return Err("快捷鍵缺少主要按鍵。請按「錄製」重新設定。".into());
    }
    let mut flags = 0_u64;
    for modifier in modifiers {
        match modifier.as_str() {
            "cmd" | "command" | "meta" => flags |= MAC_FLAG_COMMAND,
            "ctrl" | "control" => flags |= MAC_FLAG_CONTROL,
            "alt" | "option" => flags |= MAC_FLAG_OPTION,
            "shift" => flags |= MAC_FLAG_SHIFT,
            other => return Err(format!("macOS 不支援修飾鍵：{other}。請重新錄製。")),
        }
    }
    let key_code =
        mac_key_code(&key).ok_or_else(|| format!("macOS 尚未支援按鍵：{key}。請重新錄製。"))?;
    Ok((key_code, flags))
}

#[cfg(target_os = "macos")]
fn mac_post_key(key_code: u16, flags: u64) -> Result<(), String> {
    unsafe {
        let key_down = CGEventCreateKeyboardEvent(std::ptr::null_mut(), key_code, true);
        let key_up = CGEventCreateKeyboardEvent(std::ptr::null_mut(), key_code, false);
        if key_down.is_null() || key_up.is_null() {
            if !key_down.is_null() {
                CFRelease(key_down);
            }
            if !key_up.is_null() {
                CFRelease(key_up);
            }
            return Err("macOS 無法建立鍵盤事件。".into());
        }
        CGEventSetFlags(key_down, flags);
        CGEventSetFlags(key_up, flags);
        CGEventPost(0, key_down);
        thread::sleep(Duration::from_millis(35));
        CGEventPost(0, key_up);
        CFRelease(key_down);
        CFRelease(key_up);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn mac_send(shortcut: &str, target_app: &str) -> Result<(), String> {
    let (key_code, flags) = mac_shortcut(shortcut)?;
    mac_focus(target_app)?;
    thread::sleep(Duration::from_millis(450));
    mac_post_key(key_code, flags)
}

#[cfg(target_os = "windows")]
fn powershell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(target_os = "windows")]
fn windows_send_key(shortcut: &str) -> Result<String, String> {
    let (modifiers, key) = split_shortcut(shortcut);
    if key.is_empty() {
        return Err("快捷鍵缺少主要按鍵。請按「錄製」重新設定。".into());
    }
    let mut prefix = String::new();
    for modifier in modifiers {
        match modifier.as_str() {
            "ctrl" | "control" => prefix.push('^'),
            "alt" | "option" => prefix.push('%'),
            "shift" => prefix.push('+'),
            "cmd" | "command" | "meta" | "win" => {
                return Err(
                    "Windows 快捷鍵請使用 Ctrl，不要使用 macOS 的 Command。請重新錄製。".into(),
                )
            }
            other => return Err(format!("Windows 不支援修飾鍵：{other}。請重新錄製。")),
        }
    }
    let send_key = match key.as_str() {
        "enter" | "return" => "{ENTER}".into(),
        "esc" | "escape" => "{ESC}".into(),
        "backspace" => "{BACKSPACE}".into(),
        "delete" | "del" => "{DELETE}".into(),
        "insert" => "{INSERT}".into(),
        "space" => " ".into(),
        "tab" => "{TAB}".into(),
        "left" => "{LEFT}".into(),
        "right" => "{RIGHT}".into(),
        "up" => "{UP}".into(),
        "down" => "{DOWN}".into(),
        "home" => "{HOME}".into(),
        "end" => "{END}".into(),
        "pageup" => "{PGUP}".into(),
        "pagedown" => "{PGDN}".into(),
        key if key.starts_with('f') && key[1..].parse::<u8>().is_ok() => {
            format!("{{{}}}", key.to_uppercase())
        }
        "+" => "{+}".into(),
        "^" => "{^}".into(),
        "%" => "{%}".into(),
        "~" => "{~}".into(),
        "(" => "{(}".into(),
        ")" => "{)}".into(),
        "[" => "{[}".into(),
        "]" => "{]}".into(),
        "{" => "{{}".into(),
        "}" => "{}}".into(),
        key if key.chars().count() == 1 => key.into(),
        _ => return Err(format!("Windows 尚未支援按鍵：{key}。請重新錄製。")),
    };
    Ok(format!("{prefix}{send_key}"))
}

#[cfg(target_os = "windows")]
fn windows_script(target_process: &str, send_keys: Option<&str>) -> String {
    let requested = target_process.trim().trim_end_matches(".exe");
    let requested = if requested.is_empty() {
        "Nuendo"
    } else {
        requested
    };
    let requested = powershell_quote(requested);
    let send_block = send_keys
        .map(|keys| {
            format!(
                "Start-Sleep -Milliseconds 350\n[System.Windows.Forms.SendKeys]::SendWait({})",
                powershell_quote(keys)
            )
        })
        .unwrap_or_default();
    format!(
        r#"Add-Type -AssemblyName System.Windows.Forms
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class NativeWin {{
  [DllImport("user32.dll")][return: MarshalAs(UnmanagedType.Bool)] public static extern bool SetForegroundWindow(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
}}
"@
$requested = REQUESTED
$want = ($requested -replace '[^A-Za-z0-9]', '').ToLowerInvariant()
if ([string]::IsNullOrWhiteSpace($want)) {{ $want = 'nuendo' }}
$p = Get-Process | Where-Object {{
  $_.MainWindowHandle -ne 0 -and (
    (($_.ProcessName -replace '[^A-Za-z0-9]', '').ToLowerInvariant() -eq $want) -or
    (($_.ProcessName -replace '[^A-Za-z0-9]', '').ToLowerInvariant().StartsWith($want)) -or
    $_.ProcessName -like 'Nuendo*' -or
    ($_.MainWindowTitle -and $_.MainWindowTitle -like '*Nuendo*')
  )
}} | Sort-Object MainWindowHandle -Descending | Select-Object -First 1
if ($null -eq $p) {{ throw "找不到 Nuendo process：$requested。請先開啟 Nuendo，或在設定中修改 Windows Nuendo Process。" }}
[NativeWin]::ShowWindow($p.MainWindowHandle, 9) | Out-Null
if (-not [NativeWin]::SetForegroundWindow($p.MainWindowHandle)) {{ throw 'Windows 無法切換到 Nuendo。請確認 Nuendo 與本 App 使用相同權限層級。' }}
SEND_BLOCK
Write-Output ("$($p.ProcessName)|$($p.MainWindowTitle)")"#,
    )
    .replace("REQUESTED", &requested)
    .replace("SEND_BLOCK", &send_block)
}

#[cfg(target_os = "windows")]
fn run_powershell(script: &str) -> Result<String, String> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .map_err(|error| format!("無法啟動 Windows PowerShell：{error}"))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(if stderr.is_empty() {
        "Windows 無法執行快捷鍵。請確認 Nuendo 已開啟。".into()
    } else {
        stderr
    })
}

#[cfg(target_os = "windows")]
fn win_focus(target_process: &str) -> Result<String, String> {
    run_powershell(&windows_script(target_process, None))
}

#[cfg(target_os = "windows")]
fn win_send(shortcut: &str, target_process: &str) -> Result<(), String> {
    let keys = windows_send_key(shortcut)?;
    run_powershell(&windows_script(target_process, Some(&keys))).map(|_| ())
}

#[tauri::command]
#[allow(unused_variables)]
fn check_nuendo(target_app_mac: String, target_process_windows: String) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        return mac_focus(&target_app_mac);
    }
    #[cfg(target_os = "windows")]
    {
        return win_focus(&target_process_windows);
    }
    #[allow(unreachable_code)]
    Err("目前只支援 macOS / Windows".into())
}

#[tauri::command]
#[allow(unused_variables)]
fn send_shortcut(
    shortcut: String,
    target_app_mac: String,
    target_process_windows: String,
) -> Result<(), String> {
    if shortcut.trim().is_empty() {
        return Err("快捷鍵尚未設定。請按「錄製」設定後再執行。".into());
    }
    #[cfg(target_os = "macos")]
    {
        return mac_send(&shortcut, &target_app_mac);
    }
    #[cfg(target_os = "windows")]
    {
        return win_send(&shortcut, &target_process_windows);
    }
    #[allow(unreachable_code)]
    Err("目前只支援 macOS / Windows".into())
}

#[tauri::command]
fn set_always_on_top(window: tauri::WebviewWindow, enabled: bool) -> Result<(), String> {
    window
        .set_always_on_top(enabled)
        .map_err(|error| format!("無法設定永遠置頂：{error}"))
}

#[tauri::command]
fn open_accessibility_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let status = Command::new("/usr/bin/open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .status()
            .map_err(|error| format!("無法開啟輔助使用設定：{error}"))?;
        if status.success() {
            return Ok(());
        }
        return Err("無法開啟輔助使用設定。".into());
    }
    #[allow(unreachable_code)]
    Err("此功能只適用於 macOS。".into())
}

#[cfg(test)]
mod tests {
    use super::split_shortcut;

    #[test]
    fn splits_recorded_shortcut() {
        let (modifiers, key) = split_shortcut("cmd+shift+z");
        assert_eq!(modifiers, vec!["cmd", "shift"]);
        assert_eq!(key, "z");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn maps_macos_shortcuts_to_native_codes() {
        let (key, flags) = super::mac_shortcut("cmd+shift+z").unwrap();
        assert_eq!(key, 6);
        assert_eq!(flags, super::MAC_FLAG_COMMAND | super::MAC_FLAG_SHIFT);
        assert_eq!(super::mac_shortcut("f1").unwrap().0, 122);
        assert!(super::mac_shortcut("cmd+unsupported").is_err());
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            check_nuendo,
            send_shortcut,
            set_always_on_top,
            open_accessibility_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running Nuendo Conform Helper");
}
