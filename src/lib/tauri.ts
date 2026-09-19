import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
export async function copyText(text:string){if(!text)return;try{await writeText(text)}catch{await navigator.clipboard.writeText(text)}}
export async function sendNuendoShortcut(shortcut:string,targetAppMac:string,targetProcessWindows:string){if(!shortcut.trim())throw new Error("此功能尚未設定快捷鍵。");await invoke("send_shortcut",{shortcut,targetAppMac,targetProcessWindows})}
export async function checkNuendoConnection(targetAppMac:string,targetProcessWindows:string){return await invoke<string>("check_nuendo",{targetAppMac,targetProcessWindows})}
