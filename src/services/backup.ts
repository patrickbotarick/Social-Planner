import {invoke} from '@tauri-apps/api/core';
export interface BackupFile {path:string;bytes:number;sha256:string}
export interface BackupReport {path:string;manifestSha256:string;publications:number;media:number;manifest:{kind:string;format:number;id:string;appVersion:string;schemaVersion:number;createdAt:string;files:BackupFile[]}}
export interface BackupInspection {ticket:string;report:BackupReport}
export interface RestoreReport {recoveryPath:string;previousDirectory:string;restartRequired?:boolean}
export const createBackup=()=>invoke<BackupReport|null>('backup_create');
export const inspectBackup=()=>invoke<BackupInspection|null>('backup_inspect');
export const restoreBackup=(ticket:string,confirmed:boolean)=>invoke<RestoreReport>('backup_restore',{ticket,confirmed});
