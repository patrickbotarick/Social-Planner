import { useCallback, useEffect, useMemo, useState } from 'react';
import { PageHeader } from '../components/PageHeader';
import { Button } from '../components/ui/Button';
import { SearchField, Select } from '../components/ui/Controls';
import { Badge, Card, EmptyState, Toolbar } from '../components/ui/Presentation';
import { ConfirmDialog } from '../components/ui/Modal';
import { useToast } from '../components/ui/Toast';
import { useEditorialQuery } from '../hooks/useEditorialQuery';
import { deletePublication, duplicatePublication, editorialError, getPublication, listPublications } from '../services/editorial';
import { desktopAvailable } from '../services/desktop';
import { formatLabels, sourceLabels, statusLabels } from '../types/editorial';
import type { LibraryQuery, Publication, PublicationFormat, PublicationSource, EditorialStatus } from '../types/editorial';
import { editorialDate, MediaPreview } from '../features/editorial/MediaPreview';

export function Library() {
  const [search, setSearch] = useState(''); const [debounced, setDebounced] = useState('');
  const [format, setFormat] = useState<PublicationFormat | null>(null); const [source, setSource] = useState<PublicationSource | null>(null); const [status, setStatus] = useState<EditorialStatus | null>(null);
  const [sort, setSort] = useState<LibraryQuery['sort']>('recent'); const [offset, setOffset] = useState(0);
  const [busy, setBusy] = useState(false); const [deletion, setDeletion] = useState<Publication | null>(null); const notify = useToast(); const native = desktopAvailable();
  useEffect(() => { const timeout = window.setTimeout(() => { setDebounced(search); setOffset(0); }, 200); return () => { window.clearTimeout(timeout); }; }, [search]);
  const query = useMemo<LibraryQuery>(() => ({ search: debounced, format, source, status, sort, offset, limit: 24 }), [debounced,format,source,status,sort,offset]);
  const request = useCallback(() => listPublications(query), [query]); const { state, refresh } = useEditorialQuery(request);
  const duplicate = async (id: string) => { setBusy(true); try { await duplicatePublication(id); notify('Cópia criada como rascunho.'); await refresh(); } catch (e) { notify(editorialError(e),'danger'); } finally { setBusy(false); } };
  const askDelete = async (id: string) => { setBusy(true); try { setDeletion(await getPublication(id)); } catch (e) { notify(editorialError(e),'danger'); } finally { setBusy(false); } };
  const remove = async () => { if (!deletion || busy) return; setBusy(true); try { await deletePublication(deletion.id,deletion.version); setDeletion(null); notify('Publicação excluída da biblioteca.'); if (state.kind === 'ready' && state.data.items.length === 1 && offset > 0) setOffset(Math.max(0,offset-24)); else await refresh(); } catch (e) { notify(editorialError(e),'danger'); } finally { setBusy(false); } };
  const change = (update: () => void) => { update(); setOffset(0); };
  return <><PageHeader title="Biblioteca editorial" description="Seu acervo local de textos, imagens e vídeos." actions={<Button disabled={!native || busy} onClick={() => { window.location.hash = '#/nova-publicacao'; }}>Nova publicação</Button>} />
    <Toolbar label="Filtros da biblioteca"><SearchField label="Pesquisar publicações" value={search} maxLength={200} placeholder="Pesquisar título ou legenda" disabled={!native} onChange={e => { setSearch(e.target.value); }} />
      <Select label="Formato" hideLabel value={format ?? ''} disabled={!native} onChange={e => { change(() => { setFormat(e.target.value as PublicationFormat || null); }); }}><option value="">Todos os formatos</option>{Object.entries(formatLabels).map(([value,label]) => <option value={value} key={value}>{label}</option>)}</Select>
      <Select label="Origem" hideLabel value={source ?? ''} disabled={!native} onChange={e => { change(() => { setSource(e.target.value as PublicationSource || null); }); }}><option value="">Todas as origens</option><option value="manual">Manual</option><option value="trello">Trello</option></Select>
      <Select label="Estado editorial" hideLabel value={status ?? ''} disabled={!native} onChange={e => { change(() => { setStatus(e.target.value as EditorialStatus || null); }); }}><option value="">Todos os estados</option>{Object.entries(statusLabels).map(([value,label]) => <option value={value} key={value}>{label}</option>)}</Select>
      <Select label="Ordenação" hideLabel value={sort} disabled={!native} onChange={e => { change(() => { setSort(e.target.value as LibraryQuery['sort']); }); }}><option value="recent">Mais recentes</option><option value="oldest">Mais antigos</option><option value="title">Título</option><option value="updated">Última edição</option></Select>
    </Toolbar>
    {state.kind === 'loading' && <p role="status">Carregando biblioteca…</p>}
    {state.kind === 'browser' && <Card><EmptyState icon="biblioteca" title="Sua biblioteca está no aplicativo desktop" description="Abra a janela Windows para cadastrar e consultar os dados locais. Esta prévia não cria publicações." /></Card>}
    {state.kind === 'error' && <Card><div className="card-body"><p role="alert">{state.message}</p><Button variant="secondary" onClick={() => { void refresh(); }}>Tentar novamente</Button></div></Card>}
    {state.kind === 'ready' && <><p className="library-result-count" role="status">{state.data.total} {state.data.total === 1 ? 'publicação encontrada' : 'publicações encontradas'}</p>{state.data.items.length ? <div className="editorial-publication-grid">{state.data.items.map(item => <Card key={item.id} className="publication-card">{item.primaryMedia && <a href={'#/biblioteca/'+item.id} tabIndex={-1} aria-hidden="true"><MediaPreview media={item.primaryMedia} thumbnail /></a>}<div className="publication-card-body"><div className="publication-card-meta"><span>{formatLabels[item.format]} · {sourceLabels[item.source]}</span><Badge>{statusLabels[item.editorialStatus]}</Badge></div><h2><a href={'#/biblioteca/'+item.id}>{item.title}</a></h2><p className="small muted">{item.mediaCount} {item.mediaCount === 1 ? 'mídia' : 'mídias'} · {editorialDate(item.updatedAt)}</p><div className="publication-card-actions"><Button variant="secondary" size="small" disabled={busy} onClick={() => { window.location.hash = '#/biblioteca/'+item.id; }} aria-label={'Abrir: '+item.title}>Abrir</Button><Button variant="ghost" size="small" disabled={busy || item.source !== 'manual' || item.editorialStatus !== 'draft'} onClick={() => { window.location.hash = '#/biblioteca/'+item.id+'/editar'; }} aria-label={'Editar: '+item.title}>Editar</Button><Button variant="ghost" size="small" disabled={busy} onClick={() => { void duplicate(item.id); }} aria-label={'Duplicar: '+item.title}>Duplicar</Button><Button variant="ghost" size="small" disabled={busy} onClick={() => { void askDelete(item.id); }} aria-label={'Excluir: '+item.title}>Excluir</Button></div></div></Card>)}</div> : <Card><EmptyState icon="biblioteca" title={debounced || format || source || status ? 'Nenhuma publicação corresponde aos filtros' : 'Crie sua primeira publicação'} description={debounced || format || source || status ? 'Altere a pesquisa ou remova filtros para consultar outros conteúdos.' : 'Cadastre uma imagem, um vídeo ou um carrossel. Textos e arquivos ficarão neste computador.'} action={<Button onClick={() => { window.location.hash = '#/nova-publicacao'; }}>Nova publicação</Button>} /></Card>}
      <div className="library-pagination"><Button variant="secondary" size="small" disabled={offset === 0 || busy} onClick={() => { setOffset(Math.max(0,offset-24)); }}>Página anterior</Button><span className="small muted">Página {Math.floor(offset/24)+1}</span><Button variant="secondary" size="small" disabled={offset+24 >= state.data.total || busy} onClick={() => { setOffset(offset+24); }}>Próxima página</Button></div>
    </>}
    <ConfirmDialog open={!!deletion} title="Excluir publicação?" description={`“${deletion?.title ?? ''}” deixará de aparecer na biblioteca. Os arquivos serão preservados, inclusive os compartilhados com outras publicações.`} confirmLabel={busy ? 'Excluindo…' : 'Excluir publicação'} onConfirm={() => { void remove(); }} onClose={() => { if (!busy) setDeletion(null); }} />
  </>;
}
