import { describe, expect, it, vi } from 'vitest';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { describeDesktopError, desktopAvailable, getFoundationStatus } from '../src/services/desktop';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(), isTauri: vi.fn() }));

describe('Contrato IPC', () => {
  it('consulta somente o comando de diagnóstico, sem SQL ou caminhos de entrada', async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockResolvedValue({ schemaVersion: 1 });
    expect(desktopAvailable()).toBe(true);
    await expect(getFoundationStatus()).resolves.toEqual({ schemaVersion: 1 });
    expect(invoke).toHaveBeenCalledWith('foundation_status');
  });
  it.each([null, 'secret', new Error('secret'), { code: 'UNKNOWN', message: 'secret' }])('não expõe erros desconhecidos: %s', (error) => {
    expect(describeDesktopError(error)).not.toContain('secret');
    expect(describeDesktopError(error)).toContain('Não foi possível consultar');
  });
});
