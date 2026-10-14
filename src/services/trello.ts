import { invoke } from '@tauri-apps/api/core';
import type { BoardPreview, ConnectionStatus, DownloadLimits, ImportHistory, ImportJob, SourceInfo } from '../types/trello';
export const trelloStatus = () => invoke<ConnectionStatus>('trello_status');
export const trelloConnect = (key: string, token: string) => invoke<ConnectionStatus>('trello_connect', { key, token });
export const trelloTest = () => invoke<ConnectionStatus>('trello_test');
export const trelloDisconnect = () => invoke<void>('trello_disconnect');
export const trelloAuthorizationUrl = (key: string) => invoke<string>('trello_authorization_url', { key });
export const trelloPreview = (url: string) => invoke<BoardPreview>('trello_preview', { url });
export const trelloHistory = () => invoke<ImportHistory>('trello_history');
export const trelloImport = (previewId: string, selected: string[], limits: DownloadLimits) => invoke<ImportJob>('trello_import', { previewId, selected, limits });
export const trelloCancel = () => invoke<void>('trello_cancel');
export const trelloSource = (publicationId: string) => invoke<SourceInfo | null>('trello_source', { publicationId });
export function trelloError(e: unknown): string {
  if (typeof e === 'object' && e !== null && 'code' in e && 'message' in e && typeof e.code === 'string' && e.code.startsWith('TRELLO_') && typeof e.message === 'string') return e.message;
  return 'Não foi possível concluir a operação. Verifique a conexão e o armazenamento local e tente novamente.';
}
export const importStates: Record<string, string> = { running: 'Em andamento', completed: 'Concluída', partial: 'Concluída com falhas', cancelled: 'Cancelada', interrupted: 'Interrompida', failed: 'Falhou' };
