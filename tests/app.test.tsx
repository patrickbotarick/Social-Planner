import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from '../src/app/App';
import { sections } from '../src/app/navigation';
import { desktopAvailable, getFoundationStatus } from '../src/services/desktop';
import type { FoundationStatus } from '../src/types/foundation';
import type * as DesktopService from '../src/services/desktop';

vi.mock('../src/services/desktop', async (importOriginal) => {
  const original = await importOriginal<typeof DesktopService>();
  return { ...original, desktopAvailable: vi.fn(), getFoundationStatus: vi.fn() };
});

const status: FoundationStatus = {
  schemaVersion: 1,
  databasePath: 'C:\\Users\\teste\\AppData\\Roaming\\br.com.gmad.socialplanner\\planner.sqlite3',
  integrity: 'ok',
  settings: { timezone: 'America/Sao_Paulo', createdAt: '2026-10-13T12:00:00Z', updatedAt: '2026-10-13T12:00:00Z' },
};

beforeEach(() => { vi.mocked(desktopAvailable).mockReturnValue(false); });

describe('Fundação editorial', () => {
  it('identifica a prévia web sem inventar conexão com SQLite', async () => {
    render(<App />);
    expect(await screen.findByText('Você está na prévia da interface.')).toBeVisible();
    expect(getFoundationStatus).not.toHaveBeenCalled();
    expect(screen.queryByText(/Banco conectado/)).not.toBeInTheDocument();
  });

  it('navega por todas as áreas e mantém ações futuras desabilitadas', async () => {
    const user = userEvent.setup();
    render(<App />);
    for (const section of sections.slice(1)) {
      await user.click(screen.getByRole('link', { name: section.label }));
      expect(await screen.findByRole('heading', { level: 1, name: section.label })).toBeVisible();
      expect(screen.getByRole('link', { name: section.label })).toHaveAttribute('aria-current', 'page');
      if (section.id !== 'configuracoes') {
        expect(screen.getByRole('button', { name: 'Disponível em uma próxima fase' })).toBeDisabled();
        expect(screen.getByText(`Previsto para a Fase ${section.phase}`)).toBeVisible();
      }
    }
  });

  it('responde a mudanças de rota e recupera rotas desconhecidas', async () => {
    render(<App />);
    await act(async () => { window.location.hash = '#/calendario'; });
    expect(await screen.findByRole('heading', { level: 1, name: 'Calendário mensal' })).toBeVisible();
    await act(async () => { window.location.hash = '#/desconhecida'; });
    expect(await screen.findByRole('heading', { level: 1, name: 'Dashboard' })).toBeVisible();
  });

  it('mostra o estado retornado pelo desktop e permite nova verificação', async () => {
    vi.mocked(desktopAvailable).mockReturnValue(true);
    vi.mocked(getFoundationStatus).mockResolvedValue(status);
    const user = userEvent.setup();
    render(<App />);
    expect(await screen.findByText('Banco conectado. Verificação de integridade concluída.')).toBeVisible();
    expect(screen.getByText(status.databasePath)).toBeVisible();
    expect(screen.getByText('America/Sao_Paulo')).toBeVisible();
    await user.click(screen.getByRole('button', { name: 'Verificar novamente' }));
    await waitFor(() => { expect(getFoundationStatus).toHaveBeenCalledTimes(2); });
  });

  it('mostra erro de integridade, protege detalhes e permite recuperação', async () => {
    vi.mocked(desktopAvailable).mockReturnValue(true);
    vi.mocked(getFoundationStatus).mockRejectedValueOnce({ code: 'DATABASE_INTEGRITY', message: 'detalhe sensível' }).mockResolvedValue(status);
    const user = userEvent.setup();
    render(<App />);
    expect(await screen.findByRole('alert')).toHaveTextContent('A verificação do banco falhou');
    expect(screen.queryByText('detalhe sensível')).not.toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Verificar novamente' }));
    expect(await screen.findByText('Banco conectado. Verificação de integridade concluída.')).toBeVisible();
  });

  it('mantém a verificação desabilitada enquanto o comando está pendente', () => {
    vi.mocked(desktopAvailable).mockReturnValue(true);
    vi.mocked(getFoundationStatus).mockImplementation(() => new Promise(() => {}));
    render(<App />);
    expect(screen.getByRole('button', { name: 'Verificar novamente' })).toBeDisabled();
    expect(screen.getByRole('status')).toHaveTextContent('Verificando a conexão');
  });
});
