import { useState } from 'react';
import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { Tabs } from '../src/components/ui/Tabs';
import { ConfirmDialog, Modal } from '../src/components/ui/Modal';
import { Button, IconButton } from '../src/components/ui/Button';
import { Input } from '../src/components/ui/Controls';
import { Tooltip } from '../src/components/ui/Tooltip';
import { ToastProvider, useToast } from '../src/components/ui/Toast';
import { Calendar } from '../src/pages/Calendar';
import { dateKey, monthDays, todayInTimezone } from '../src/utils/calendar';

describe('Controles e acessibilidade', () => {
  it('associa erro e rótulo ao campo, sem depender de placeholder', () => {
    render(<Input label="Título" error="Informe o título" />);
    const input = screen.getByRole('textbox', { name: 'Título' });
    expect(input).toHaveAttribute('aria-invalid', 'true');
    expect(input).toHaveAccessibleDescription('Informe o título');
  });
  it('desabilita a ação real de um botão indisponível', async () => {
    const handler = vi.fn();
    render(<Button disabled onClick={handler}>Salvar</Button>);
    await userEvent.setup().click(screen.getByRole('button', { name: 'Salvar' }));
    expect(handler).not.toHaveBeenCalled();
  });
  it('navega tabs com setas, Home e End, com foco no tab ativo', async () => {
    render(<Tabs label="Configurações" items={[{ id: 'one', label: 'Um', content: 'Painel um' }, { id: 'two', label: 'Dois', content: 'Painel dois' }]} />);
    const user = userEvent.setup();
    await user.tab();
    await user.keyboard('{ArrowRight}');
    expect(screen.getByRole('tab', { name: 'Dois' })).toHaveFocus();
    expect(screen.getByRole('tabpanel')).toHaveTextContent('Painel dois');
    await user.keyboard('{Home}');
    expect(screen.getByRole('tab', { name: 'Um' })).toHaveAttribute('aria-selected', 'true');
    await user.keyboard('{End}');
    expect(screen.getByRole('tab', { name: 'Dois' })).toHaveAttribute('tabindex', '0');
  });
  it('abre diálogo rotulado e devolve foco ao fechar, inclusive via cancel', async () => {
    function Example() {
      const [open, setOpen] = useState(false);
      return <><Button onClick={() => { setOpen(true); }}>Ajuda</Button><Modal open={open} title="Sobre" onClose={() => { setOpen(false); }}><p>Conteúdo do diálogo.</p></Modal></>;
    }
    render(<Example />);
    const user = userEvent.setup();
    const trigger = screen.getByRole('button', { name: 'Ajuda' });
    await user.click(trigger);
    expect(screen.getByRole('dialog', { name: 'Sobre' })).toBeVisible();
    expect(screen.getByRole('button', { name: 'Fechar diálogo' })).toHaveFocus();
    await user.tab();
    expect(screen.getByRole('button', { name: 'Fechar diálogo' })).toHaveFocus();
    await user.tab({ shift: true });
    expect(screen.getByRole('button', { name: 'Fechar diálogo' })).toHaveFocus();
    fireEvent(screen.getByRole('dialog'), new Event('cancel', { bubbles: false, cancelable: true }));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(trigger).toHaveFocus();
  });
  it('confirma apenas por ação explícita; cancelar não aciona confirmação', async () => {
    const confirm = vi.fn(); const close = vi.fn();
    render(<ConfirmDialog open title="Confirmar ação" description="Exemplo isolado em teste." confirmLabel="Confirmar" onConfirm={confirm} onClose={close} />);
    const user = userEvent.setup();
    await user.click(screen.getByRole('button', { name: 'Cancelar' }));
    expect(close).toHaveBeenCalledOnce();
    expect(confirm).not.toHaveBeenCalled();
    await user.click(screen.getByRole('button', { name: 'Confirmar' }));
    expect(confirm).toHaveBeenCalledOnce();
  });
  it('mostra tooltip por teclado e permite dispensá-lo com Escape', async () => {
    render(<Tooltip text="Orientação"><IconButton label="Ajuda">?</IconButton></Tooltip>);
    const user = userEvent.setup();
    await user.tab();
    expect(screen.getByRole('button')).toHaveAccessibleDescription('Orientação');
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  });
  it('anuncia e permite dispensar o feedback de uma ação', async () => {
    function Example() { const notify = useToast(); return <Button onClick={() => { notify('Caminho copiado.'); }}>Copiar</Button>; }
    render(<ToastProvider><Example /></ToastProvider>);
    const user = userEvent.setup();
    await user.click(screen.getByRole('button', { name: 'Copiar' }));
    expect(screen.getByRole('status')).toHaveTextContent('Caminho copiado.');
    await user.click(screen.getByRole('button', { name: 'Fechar aviso' }));
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });
});

describe('Calendário visual, sem agendamento', () => {
  it('respeita o fuso na fronteira UTC do dia', () => {
    const instant = new Date('2026-10-14T01:30:00Z');
    expect(todayInTimezone('America/Sao_Paulo', instant)).toEqual({ year: 2026, month: 9, day: 13 });
    expect(todayInTimezone('UTC', instant).day).toBe(14);
  });
  it('inclui fevereiro bissexto e dias adjacentes sem duplicar datas', () => {
    const days = monthDays(2024, 1);
    expect(days.filter((day) => day.month === 1)).toHaveLength(29);
    expect(dateKey(days[0]!)).toBe('2024-01-28');
    expect(dateKey(days[41]!)).toBe('2024-03-09');
    expect(new Set(days.map(dateKey)).size).toBe(42);
  });
  it('navega meses, retorna ao mês atual e mantém agendamento indisponível', async () => {
    render(<Calendar timezone="America/Sao_Paulo" />);
    const title = screen.getByRole('heading', { level: 2 }).textContent;
    const user = userEvent.setup();
    await user.click(screen.getByRole('button', { name: 'Próximo mês' }));
    expect(screen.getByRole('heading', { level: 2 }).textContent).not.toBe(title);
    await user.click(screen.getByRole('button', { name: 'Hoje' }));
    expect(screen.getByRole('heading', { level: 2 }).textContent).toBe(title);
    expect(screen.getByRole('button', { name: 'Planejar publicação' })).toBeDisabled();
    expect(screen.getAllByRole('listitem')).toHaveLength(42);
  });
});
