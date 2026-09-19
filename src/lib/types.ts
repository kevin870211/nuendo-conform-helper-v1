export type ItemType="move"|"slow"|"reuse"|"new_shot"|"os"|"sound"|"replace"|"freeze"|"note";
export type ItemStatus="pending"|"done"|"skipped";
export interface TimeRange{in:string;out:string}
export interface ConformItem{id:string;order:number;type:ItemType;status:ItemStatus;destination?:TimeRange;pointTc?:string;sourceVersion?:string;source?:TimeRange;note:string;rawText:string;tags:string[];warnings:string[]}
export interface ShortcutMap{cut:string;copy:string;paste:string;undo:string;redo:string;insertSilence:string;splitLoop:string;selectInLoop:string}
export interface AppSettings{platformMode:"auto"|"mac"|"windows";targetAppMac:string;targetProcessWindows:string;nameTemplate:string;fps:number;fpsConfirmed:boolean;alwaysOnTop:boolean;shortcutsMac:ShortcutMap;shortcutsWindows:ShortcutMap}
export interface ProjectState{name:string;items:ConformItem[];currentId?:string;settings:AppSettings;importedAt?:string}
