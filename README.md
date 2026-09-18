# Nuendo Conform Helper V1

跨平台（Windows / macOS）的 Nuendo Conform 輔助工具原型。

## 已實作

- 匯入 DOCX，解析新版 TC、來源版本、來源 TC、單點 TC 與備註
- NORMAL / SLOW / REUSE / NEW SHOT / OS / SFX / REPLACE / FREEZE 分類
- 單一 TC 一鍵複製
- 整串 TC 一鍵複製，例如 `00:02:56:05~00:03:09:07`
- Loop 命名模板與「複製整串文字」
- 複製原始整行、備註、版本＋整段
- 工作清單、搜尋、完成進度與自動保存
- Alt/Option+1、+2、+3 全域 Quick Copy
- Nuendo 快捷鍵面板
- macOS：AppleScript 切回 Nuendo 並送快捷鍵
- Windows：PowerShell / SendKeys 切回 Nuendo 並送快捷鍵

Parser 針對目前調整文件格式，例如：

`00:00:56:18~00:01:09:20 是 0512 版 R1 00:02:56:05~00:03:09:07`

`00:10:05:07 新增颱風廣播`

`00:17:42:21~00:17:45:20 是新增鏡頭`

若原始文件出現像 `00;11:14:10` 的分號時間碼，程式會暫時正規化為冒號並顯示人工確認警告。

## 開發 / 執行

需求：Node.js 20+、Rust stable、Tauri 2 prerequisites。

```bash
npm install
npm run tauri dev
```

正式打包：

```bash
npm run tauri build
```

## macOS

第一次使用快捷鍵控制，需要到「系統設定 → 隱私權與安全性 → 輔助使用」允許本工具控制鍵盤。預設 App 名稱是 `Nuendo 14`，可在設定修改。

## Windows

預設尋找 Process 名稱開頭為 `Nuendo` 的程式，可在設定修改。

### GitHub Actions Windows 建置

專案已附上 `.github/workflows/build-windows.yml`。將專案推送到 GitHub 後，可在 GitHub 的 Actions 分頁手動執行 `Build Windows installers`，或推送到 `main`／`master` 時自動建置。完成後到該次 workflow 的 Artifacts 下載 Windows x64 的 `.msi` 與 NSIS `-setup.exe`。

此流程使用 GitHub 的 Windows runner，不需要在 Parallels 內安裝 Node.js、Rust 或 Visual Studio Build Tools。

## Loop 命名模板

預設 `{source_range}`。可用：`{source_version}`、`{source_in}`、`{source_out}`、`{source_range}`、`{new_in}`、`{new_out}`、`{new_range}`、`{note}`、`{type}`。

例如 `{source_version}_{source_range}` 可得到：

`0512_R1_00:02:56:05~00:03:09:07`

## V1 暫不全自動搬移

V1 不直接自動填 Left / Right Locator，也不執行整套「設定 Locator → Split Loop → Select In Loop → Cut/Copy → 跳新版 → Paste」。先把低風險重複動作穩定下來，下一版再用實機 Nuendo 驗證 Locator/Macro。
