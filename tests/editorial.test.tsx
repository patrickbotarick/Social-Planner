import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from '../src/app/App';
import { PublicationEditor } from '../src/features/editorial/PublicationEditor';
import { Library } from '../src/pages/Library';
import { PublicationDetails } from '../src/pages/PublicationDetails';
import { ToastProvider } from '../src/components/ui/Toast';
import { desktopAvailable, getFoundationStatus } from '../src/services/desktop';
import * as editorial from '../src/services/editorial';
import type { MediaAsset, Publication } from '../src/types/editorial';
import type * as DesktopService from '../src/services/desktop';

vi.mock('../src/services/desktop', async original => ({ ...await original<typeof DesktopService>(), desktopAvailable: vi.fn(), getFoundationStatus: vi.fn() }));
vi.mock('../src/services/editorial', async original => ({ ...await original<typeof editorial>(), selectMedia: vi.fn(), discardMedia: vi.fn(), savePublication: vi.fn(), getPublication: vi.fn(), listPublications: vi.fn(), getEditorialCounts: vi.fn(), duplicatePublication: vi.fn(), deletePublication: vi.fn(), mediaUrl: vi.fn(() => 'data:image/png;base64,') }));
const id='00000000-0000-4000-8000-000000000001';
const first: MediaAsset = { id:'00000000-0000-4000-8000-000000000002',filename:'imagem.png',mimeType:'image/png',extension:'png',sizeBytes:2048,storagePath:'internal.png',previewPath:'controlled.png',width:24,height:16,duration:null,checksum:'a'.repeat(64),createdAt:'2026-10-13T12:00:00Z' };
const second: MediaAsset = { ...first,id:'00000000-0000-4000-8000-000000000003',filename:'segunda.png',checksum:'b'.repeat(64) };
const publication: Publication = { id,title:'Campanha manual',caption:'Linha 1 😊\n#GMAD',format:'feed',source:'manual',editorialStatus:'draft',version:1,createdAt:'2026-10-13T12:00:00Z',updatedAt:'2026-10-13T12:00:00Z',media:[first] };
beforeEach(() => {
  vi.mocked(desktopAvailable).mockReturnValue(true);
  vi.mocked(getFoundationStatus).mockResolvedValue({ schemaVersion:2,databasePath:'controlled/planner.sqlite3',integrity:'ok',settings:{timezone:'America/Sao_Paulo',createdAt:publication.createdAt,updatedAt:publication.updatedAt} });
  vi.mocked(editorial.selectMedia).mockResolvedValue({media:[first],issues:[]});
  vi.mocked(editorial.discardMedia).mockResolvedValue();
  vi.mocked(editorial.savePublication).mockResolvedValue(publication);
  vi.mocked(editorial.getPublication).mockResolvedValue(publication);
  vi.mocked(editorial.listPublications).mockResolvedValue({items:[],total:0});
  vi.mocked(editorial.getEditorialCounts).mockResolvedValue({total:3,drafts:3,pendingReview:0,approved:0});
  vi.mocked(editorial.duplicatePublication).mockResolvedValue({...publication,id:'00000000-0000-4000-8000-000000000004',title:'Campanha manual (cópia)'});
  vi.mocked(editorial.deletePublication).mockResolvedValue();
});
function editor(existing = false) {
  const onSaved=vi.fn(); const onGuard=vi.fn();
  render(<ToastProvider><PublicationEditor {...(existing ? {id} : {})} onSaved={onSaved} onCancel={vi.fn()} onGuard={onGuard} /></ToastProvider>);
  return {onSaved,onGuard};
}

describe('Formulário editorial', () => {
  it('salva título, legenda com emoji/quebra de linha e mídia pelo contrato tipado', async () => {
    const {onSaved,onGuard}=editor(); const user=userEvent.setup();
    await user.type(screen.getByRole('textbox',{name:'Título interno'}),'Campanha');
    await user.type(screen.getByRole('textbox',{name:'Legenda'}),'Linha 1 😊{enter}#GMAD');
    await user.click(screen.getByRole('button',{name:'Selecionar mídias'}));
    await user.click(screen.getByRole('button',{name:'Salvar publicação'}));
    expect(editorial.savePublication).toHaveBeenCalledWith({id:null,expectedVersion:null,title:'Campanha',caption:'Linha 1 😊\n#GMAD',format:'feed',mediaIds:[first.id]});
    expect(onSaved).toHaveBeenCalledWith(publication); expect(onGuard).toHaveBeenCalledWith(null);
  });
  it('valida título e quantidade, sem enviar salvamento inválido', async () => {
    editor(); const user=userEvent.setup(); await user.click(screen.getByRole('button',{name:'Salvar publicação'}));
    expect(screen.getByRole('alert')).toHaveTextContent('título interno');
    await user.type(screen.getByRole('textbox',{name:'Título interno'}),'Título'); await user.click(screen.getByRole('button',{name:'Salvar publicação'}));
    expect(screen.getByRole('alert')).toHaveTextContent('Selecione uma mídia'); expect(editorial.savePublication).not.toHaveBeenCalled();
  });
  it('reordena carrossel e descarta uma mídia temporária removida', async () => {
    vi.mocked(editorial.selectMedia).mockResolvedValue({media:[first,second],issues:[]}); editor(); const user=userEvent.setup();
    await user.type(screen.getByRole('textbox',{name:'Título interno'}),'Carrossel'); await user.selectOptions(screen.getByRole('combobox',{name:'Formato editorial'}),'carousel');
    await user.click(screen.getByRole('button',{name:'Selecionar mídias'})); await user.click(screen.getByRole('button',{name:'Mover para baixo: imagem.png'})); await user.click(screen.getByRole('button',{name:'Salvar publicação'}));
    expect(editorial.savePublication).toHaveBeenCalledWith(expect.objectContaining({mediaIds:[second.id,first.id],format:'carousel'}));
    await user.click(screen.getByRole('button',{name:'Remover mídia 1'})); expect(editorial.discardMedia).toHaveBeenCalledWith([second.id]);
  });
  it('reabre publicação e envia a versão esperada durante edição', async () => {
    editor(true); const user=userEvent.setup(); const title=await screen.findByDisplayValue('Campanha manual');
    expect(screen.getByRole('textbox',{name:'Legenda'})).toHaveValue(publication.caption);
    await user.clear(title); await user.type(title,'Título editado'); await user.click(screen.getByRole('button',{name:'Salvar publicação'}));
    expect(editorial.savePublication).toHaveBeenCalledWith(expect.objectContaining({id,expectedVersion:1,title:'Título editado',caption:publication.caption,mediaIds:[first.id]}));
  });
  it('mantém dados quando falha o salvamento e identifica rejeição de arquivo', async () => {
    vi.mocked(editorial.selectMedia).mockResolvedValue({media:[first],issues:[{filename:'invalido.png',message:'Arquivo danificado.'}]});
    vi.mocked(editorial.savePublication).mockRejectedValue({code:'MEDIA_IO',message:'Falha de escrita.'}); editor(); const user=userEvent.setup();
    await user.type(screen.getByRole('textbox',{name:'Título interno'}),'Não perder'); await user.click(screen.getByRole('button',{name:'Selecionar mídias'}));
    expect(screen.getByRole('alert')).toHaveTextContent('invalido.png'); await user.click(screen.getByRole('button',{name:'Salvar publicação'}));
    expect(await screen.findByText('Falha de escrita.')).toBeVisible(); expect(screen.getByRole('textbox',{name:'Título interno'})).toHaveValue('Não perder');
  });
  it('evita perda por navegação e só descarta após confirmação', async () => {
    window.history.replaceState(null,'','#/nova-publicacao'); render(<App />); const user=userEvent.setup();
    await user.type(screen.getByRole('textbox',{name:'Título interno'}),'Ainda não salvo'); await user.click(screen.getByRole('button',{name:'Selecionar mídias'}));
    await user.click(screen.getByRole('link',{name:'Biblioteca editorial'})); expect(screen.getByRole('dialog')).toHaveAccessibleName('Descartar alterações não salvas?');
    await user.click(within(screen.getByRole('dialog')).getByRole('button',{name:'Cancelar'})); expect(screen.getByRole('textbox',{name:'Título interno'})).toHaveValue('Ainda não salvo');
    expect(editorial.discardMedia).not.toHaveBeenCalledWith([first.id]); await user.click(screen.getByRole('link',{name:'Biblioteca editorial'}));
    await user.click(screen.getByRole('button',{name:'Descartar e sair'})); expect(await screen.findByRole('heading',{level:1,name:'Biblioteca editorial'})).toBeVisible(); expect(editorial.discardMedia).toHaveBeenCalledWith([first.id]);
  });
  it('protege também mudança de hash pelo histórico', async () => {
    window.history.replaceState(null,'','#/nova-publicacao'); render(<App />); const user=userEvent.setup();
    await user.type(screen.getByRole('textbox',{name:'Título interno'}),'Em edição'); await act(async () => { window.location.hash='#/dashboard'; });
    expect(await screen.findByRole('dialog')).toHaveAccessibleName('Descartar alterações não salvas?'); expect(window.location.hash).toBe('#/nova-publicacao');
    await user.click(within(screen.getByRole('dialog')).getByRole('button',{name:'Cancelar'})); expect(screen.getByRole('textbox',{name:'Título interno'})).toHaveValue('Em edição');
  });
});

describe('Biblioteca e indicadores', () => {
  it('pesquisa com atraso curto e passa filtros/paginação ao backend', async () => {
    render(<ToastProvider><Library /></ToastProvider>); const user=userEvent.setup(); await screen.findByText('Crie sua primeira publicação');
    await user.type(screen.getByRole('searchbox',{name:'Pesquisar publicações'}),'campanha'); await waitFor(() => { expect(editorial.listPublications).toHaveBeenLastCalledWith(expect.objectContaining({search:'campanha',offset:0,limit:24})); });
    await user.selectOptions(screen.getByRole('combobox',{name:'Formato'}),'reels'); await user.selectOptions(screen.getByRole('combobox',{name:'Origem'}),'manual'); await user.selectOptions(screen.getByRole('combobox',{name:'Estado editorial'}),'draft'); await user.selectOptions(screen.getByRole('combobox',{name:'Ordenação'}),'updated');
    await waitFor(() => { expect(editorial.listPublications).toHaveBeenLastCalledWith(expect.objectContaining({format:'reels',source:'manual',status:'draft',sort:'updated'})); });
    expect(screen.getByText('Nenhuma publicação corresponde aos filtros')).toBeVisible();
  });
  it('duplica e confirma exclusão com versão, atualizando a listagem', async () => {
    vi.mocked(editorial.listPublications).mockResolvedValue({items:[{id,title:publication.title,format:'feed',source:'manual',editorialStatus:'draft',createdAt:publication.createdAt,updatedAt:publication.updatedAt,mediaCount:1,primaryMedia:first}],total:1});
    render(<ToastProvider><Library /></ToastProvider>); const user=userEvent.setup(); await screen.findByRole('button',{name:'Duplicar: Campanha manual'});
    await user.click(screen.getByRole('button',{name:'Duplicar: Campanha manual'})); await waitFor(() => { expect(editorial.duplicatePublication).toHaveBeenCalledWith(id); });
    await user.click(screen.getByRole('button',{name:'Excluir: Campanha manual'})); const dialog=await screen.findByRole('dialog'); expect(editorial.deletePublication).not.toHaveBeenCalled(); await user.click(within(dialog).getByRole('button',{name:'Excluir publicação'}));
    await waitFor(() => { expect(editorial.deletePublication).toHaveBeenCalledWith(id,1); }); expect(vi.mocked(editorial.listPublications).mock.calls.length).toBeGreaterThanOrEqual(3);
  });
  it('mostra legenda completa e copia com feedback', async () => {
    const user=userEvent.setup(); render(<ToastProvider><PublicationDetails id={id} /></ToastProvider>); await screen.findByRole('heading',{name:publication.title});
    expect(screen.getByText(/Linha 1/)).toHaveTextContent('#GMAD'); await user.click(screen.getByRole('button',{name:'Copiar legenda'}));
    expect(await navigator.clipboard.readText()).toBe(publication.caption); expect(screen.getByRole('status')).toHaveTextContent('Legenda copiada.');
  });
  it('consulta novamente indicadores reais ao voltar após uma operação', async () => {
    render(<App />); const user=userEvent.setup(); await screen.findByText('Contagens reais do banco local. Publicações excluídas não são contabilizadas.');
    const summary=screen.getByRole('region',{name:'Resumo editorial'}); expect(within(summary).getByText('Total de publicações')).toBeVisible();
    await user.click(screen.getByRole('link',{name:'Biblioteca editorial'})); await screen.findByText('Crie sua primeira publicação');
    vi.mocked(editorial.getEditorialCounts).mockResolvedValue({total:4,drafts:4,pendingReview:0,approved:0}); await user.click(screen.getByRole('link',{name:'Dashboard'}));
    await waitFor(() => { expect(editorial.getEditorialCounts).toHaveBeenCalledTimes(2); }); expect(within(screen.getByRole('region',{name:'Resumo editorial'})).getAllByText('4')).toHaveLength(2);
  });
  it('apresenta negrito Trello e copia a legenda editorial integral, com os marcadores', async () => {
    const caption = '**Texto editorial 😊**\n\\#GMAD\n<script>texto inerte</script>';
    vi.mocked(editorial.getPublication).mockResolvedValue({ ...publication, source: 'trello', caption });
    const user = userEvent.setup();
    render(<ToastProvider><PublicationDetails id={id} /></ToastProvider>);
    await screen.findByRole('heading', { name: publication.title });
    expect(screen.getByText('Texto editorial 😊').tagName).toBe('STRONG');
    await user.click(screen.getByRole('button', { name: 'Copiar legenda' }));
    expect(await navigator.clipboard.readText()).toBe(caption);
    expect(editorial.savePublication).not.toHaveBeenCalled();
  });
});
