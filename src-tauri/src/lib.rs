use std::process::Command;

fn split_shortcut(shortcut: &str) -> (Vec<String>, String) {
    let mut parts: Vec<String> = shortcut
        .split('+')
        .map(|part| part.trim().to_lowercase())
        .filter(|part| !part.is_empty())
        .collect();
    let key = parts.pop().unwrap_or_default();
    (parts, key)
}

fn escape_applescript(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(target_os = "macos")]
fn run_osascript(script: &str) -> Result<String, String> {
    let output = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|error| format!("無法啟動 macOS AppleScript：{error}"))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.contains("-1743") || stderr.to_lowercase().contains("not allowed") {
        return Err("macOS 沒有允許輔助使用。請到「系統設定 → 隱私權與安全性 → 輔助使用」，允許 Nuendo Conform Helper 後重新執行。".into());
    }
    Err(if stderr.is_empty() {
        "macOS 無法執行快捷鍵。請確認 Nuendo 已開啟。".into()
    } else {
        stderr
    })
}

#[cfg(target_os = "macos")]
fn mac_target_script(target_app: &str, action: &str) -> String {
    let requested = target_app.trim().trim_end_matches(".app");
    let requested = if requested.is_empty() {
        "Nuendo"
    } else {
        requested
    };
    let requested = escape_applescript(requested);

    // Use System Events process objects so the Accessibility permission boundary is explicit.
    format!(
        r#"tell application "System Events"
set requestedName to "REQUESTED"
set candidates to every process whose name is requestedName or name is requestedName & ".app" or name contains "Nuendo"
if (count of candidates) is 0 then error "找不到 Nuendo。請先開啟 Nuendo，或在設定中修改 macOS Nuendo App 名稱。" number 1001
set targetProcess to item 1 of candidates
set frontmost of targetProcess to true
delay 0.2
ACTION
return name of targetProcess
end tell"#,
    )
    .replace("REQUESTED", &requested)
    .replace("ACTION", action)
}

#[cfg(target_os = "macos")]
fn mac_key_statement(shortcut: &str) -> Result<String, String> {
    let (modifiers, key) = split_shortcut(shortcut);
    if key.is_empty() {
        return Err("快捷鍵缺少主要按鍵。請按「錄製」重新設定。".into());
    }

    let mut apple_modifiers: Vec<&str> = Vec::new();
    for modifier in modifiers {
        match modifier.as_str() {
            "cmd" | "command" | "meta" => apple_modifiers.push("command down"),
            "ctrl" | "control" => apple_modifiers.push("control down"),
            "alt" | "option" => apple_modifiers.push("option down"),
            "shift" => apple_modifiers.push("shift down"),
            other => return Err(format!("macOS 不支援修飾鍵：{other}。請重新錄製。")),
        }
    }

    let using = if apple_modifiers.is_empty() {
        String::new()
    } else {
        format!(" using {{{}}}", apple_modifiers.join(", "))
    };

    let statement = match key.as_str() {
        "enter" | "return" => format!("key code 36{using}"),
        "esc" | "escape" => format!("key code 53{using}"),
        "backspace" | "delete" => format!("key code 51{using}"),
        "space" => format!("key code 49{using}"),
        "tab" => format!("key code 48{using}"),
        "left" => format!("key code 123{using}"),
        "right" => format!("key code 124{using}"),
        "down" => format!("key code 125{using}"),
        "up" => format!("key code 126{using}"),
        "home" => format!("key code 115{using}"),
        "end" => format!("key code 119{using}"),
        "pageup" => format!("key code 116{using}"),
        "pagedown" => format!("key code 121{using}"),
        "f1" => format!("key code 122{using}"),
        "f2" => format!("key code 120{using}"),
        "f3" => format!("key code 99{using}"),
        "f4" => format!("key code 118{using}"),
        "f5" => format!("key code 96{using}"),
        "f6" => format!("key code 97{using}"),
        "f7" => format!("key code 98{using}"),
        "f8" => format!("key code 100{using}"),
        "f9" => format!("key code 101{using}"),
        "f10" => format!("key code 109{using}"),
        "f11" => format!("key code 103{using}"),
        "f12" => format!("key code 111{using}"),
        key if key.starts_with('f') && key[1..].parse::<u8>().is_ok() => {
            format!("keystroke \"{}\"{}", escape_applescript(&key), using)
        }
        key if key.chars().count() == 1 => {
            format!("keystroke \"{}\"{}", escape_applescript(key), using)
        }
        _ => return Err(format!("macOS 尚未支援按鍵：{key}。請重新錄製。")),
    };

    Ok(statement)
}

#[cfg(target_os = "macos")]
fn mac_focus(target_app: &str) -> Result<String, String> {
    run_osascript(&mac_target_script(target_app, ""))
}

#[cfg(target_os = "macos")]
fn mac_send(shortcut: &str, target_app: &str) -> Result<(), String> {
    let action = mac_key_statement(shortcut)?;
    run_osascript(&mac_target_script(target_app, &action)).map(|_| ())
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
                "Start-Sleep -Milliseconds 200\n[System.Windows.Forms.SendKeys]::SendWait({})",
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
fn check_nuendo(target_app_mac: String, _target_process_windows: String) -> Result<String, String> {
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
fn send_shortcut(
    shortcut: String,
    target_app_mac: String,
    _target_process_windows: String,
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![check_nuendo, send_shortcut])
        .run(tauri::generate_context!())
        .expect("error while running Nuendo Conform Helper");
}
