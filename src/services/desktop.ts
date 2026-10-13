import { invoke, isTauri } from '@tauri-apps/api/core';
import type { FoundationStatus } from '../types/foundation';

export const desktopAvailable = (): boolean => isTauri();
export const getFoundationStatus = (): Promise<FoundationStatus> =>
  invoke<FoundationStatus>('foundation_status');

export function describeDesktopError(error: unknown): string {
  if (typeof error === 'object' && error !== null && 'code' in error) {
    switch (error.code) {
      case 'DATABASE_UNAVAILABLE': return 'Não foi possível abrir o banco local. Verifique o acesso ao diretório de dados e reinicie o aplicativo. Nenhum dado foi apagado.';
      case 'DATABASE_INTEGRITY': return 'A verificação do banco falhou. Preserve os arquivos e procure suporte antes de continuar.';
      case 'SCHEMA_UNSUPPORTED': return 'Este banco usa uma estrutura incompatível. Use a versão correspondente do aplicativo; não apague o banco.';
    }
  }
  return 'Não foi possível consultar os dados locais. Tente verificar novamente. Se o problema persistir, reinicie o aplicativo.';
}
