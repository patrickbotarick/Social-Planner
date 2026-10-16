import { TrelloSource } from './TrelloSource';
import { useCallback, useEffect, useRef, useState } from 'react';
import { PageHeader } from '../../components/PageHeader';
import { Button, IconButton } from '../../components/ui/Button';
import { Input, Select, Textarea } from '../../components/ui/Controls';
import { Badge, Card, EmptyState } from '../../components/ui/Presentation';
import { useToast } from '../../components/ui/Toast';
import { desktopAvailable } from '../../services/desktop';
import { discardMedia, editorialError, getPublication, savePublication, selectMedia } from '../../services/editorial';
import { formatLabels, statusLabels } from '../../types/editorial';
import type { LeaveGuard, MediaAsset, Publication, PublicationFormat } from '../../types/editorial';
import { fileSize, MediaPreview } from './MediaPreview';

interface Props { id?: string; onSaved: (publication: Publication) => void; onCancel: () => void; onGuard: (guard: LeaveGuard | null) => void }
interface Draft { title: string; caption: string; format: PublicationFormat; media: MediaAsset[] }
const empty: Draft = { title: '', caption: '', format: 'feed', media: [] };
const signature = (draft: Draft) => JSON.stringify([draft.title,draft.caption,draft.format,draft.media.map(m => m.id)]);
function validation(draft: Draft, allowEmpty = false): string | null {
  if (!draft.title.trim() || [...draft.title].length > 200) return 'Informe um título interno de até 200 caracteres.';
  if ([...draft.caption].length > 20000) return 'A legenda deve ter até 20.000 caracteres.';
  const count = draft.media.length;
  if (allowEmpty && count === 0) return null;
  if (draft.format === 'carousel' ? count < 2 || count > 20 : count !== 1) return draft.format === 'carousel' ? 'Selecione de 2 a 20 mídias para o carrossel.' : 'Selecione uma mídia para este formato.';
  if (draft.format === 'feed' && draft.media.some(m => !m.mimeType.startsWith('image/'))) return 'Feed aceita uma imagem.';
  if (draft.format === 'reels' && draft.media.some(m => !m.mimeType.startsWith('video/'))) return 'Reels aceita um vídeo MP4.';
  return null;
}
export function PublicationEditor({ id, onSaved, onCancel, onGuard }: Props) {
  const native = desktopAvailable(); const notify = useToast();
  const [draft, setDraft] = useState<Draft>(empty); const [baseline, setBaseline] = useState(signature(empty));
  const [loaded, setLoaded] = useState<Publication | null>(null);
  const [loading, setLoading] = useState(!!id); const [busy, setBusy] = useState(false); const [error, setError] = useState<string | null>(null);
  const [issues, setIssues] = useState<string[]>([]); const request = useRef(0);
  const load = useCallback(async () => {
    if (!id || !native) { setLoading(false); return; }
    const current = ++request.current; setLoading(true); setError(null);
    try { const publication = await getPublication(id); if (current !== request.current) return; const value = { title: publication.title, caption: publication.caption, format: publication.format, media: publication.media }; setLoaded(publication); setDraft(value); setBaseline(signature(value)); }
    catch (e) { if (current === request.current) setError(editorialError(e)); }
    finally { if (current === request.current) setLoading(false); }
  }, [id, native]);
  useEffect(() => { void load(); return () => { request.current += 1; }; }, [load]);
  const discard = useCallback(() => discardMedia(draft.media.map(m => m.id)), [draft.media]);

  const dirty = signature(draft) !== baseline;
  const select = async () => {
    setBusy(true); setError(null); setIssues([]);
    try {
      const result = await selectMedia(draft.format);
      const media = [...draft.media]; const discarded: string[] = []; const warnings = result.issues.map(issue => issue.filename + ': ' + issue.message);
      for (const item of result.media) {
        if (media.some(m => m.checksum === item.checksum)) { discarded.push(item.id); warnings.push(item.filename + ': este arquivo já está na publicação.'); }
        else if (media.length >= (draft.format === 'carousel' ? 20 : 1)) { discarded.push(item.id); warnings.push(item.filename + ': remova a mídia atual antes de selecionar outra.'); }
        else { media.push(item); }
      }
      await discardMedia(discarded); setDraft(current => ({ ...current, media })); setIssues(warnings);
    } catch (e) { setError(editorialError(e)); } finally { setBusy(false); }
  };
  const remove = async (media: MediaAsset) => {
    setBusy(true); setError(null);
    try { await discardMedia([media.id]); setDraft(current => ({ ...current, media: current.media.filter(m => m.id !== media.id) })); }
    catch (e) { setError(editorialError(e)); } finally { setBusy(false); }
  };
  const reorder = (index: number, delta: number) => { setDraft(current => { const media = [...current.media]; const item = media[index]; if (item) { media.splice(index,1); media.splice(index+delta,0,item); } return { ...current, media }; }); };
  const save = useCallback(async () => {
    const message = validation(draft, !!loaded && loaded.editorialStatus !== 'draft'); if (message) { setError(message); return false; }
    setBusy(true); setError(null);
    try { const publication = await savePublication({ id: loaded?.id ?? null, expectedVersion: loaded?.version ?? null, title: draft.title, caption: draft.caption, format: draft.format, mediaIds: draft.media.map(m => m.id) }); onGuard(null); setBaseline(signature(draft)); notify(loaded ? 'Alterações salvas.' : 'Publicação criada.'); onSaved(publication); return true; }
    catch (e) { setError(editorialError(e)); return false; } finally { setBusy(false); }
  }, [draft, loaded, onGuard, notify, onSaved]);
  useEffect(() => { onGuard({ dirty, busy: busy || loading, discard, save }); return () => { onGuard(null); }; }, [dirty, busy, loading, discard, save, onGuard]);
  return <><PageHeader title={id ? 'Editar publicação' : 'Nova publicação'} description="Prepare o conteúdo da unidade. O salvamento permanece neste computador." actions={<div className="page-actions"><Badge>{dirty ? 'Alterações não salvas' : loaded ? statusLabels[loaded.editorialStatus] : 'Rascunho'}</Badge><Button variant="secondary" onClick={onCancel} disabled={busy || loading}>Cancelar</Button><Button onClick={() => { void save(); }} disabled={!native || busy || loading || !!id && !loaded}>{busy ? 'Processando…' : 'Salvar publicação'}</Button></div>} />
    {!native && <p role="status" className="editorial-notice">Abra a janela desktop para cadastrar e armazenar publicações. Esta prévia não simula persistência.</p>}
    {loading ? <p role="status">Carregando publicação…</p> : id && !loaded ? <Card><div className="card-body"><p role="alert">{error ?? 'Publicação indisponível.'}</p><Button variant="secondary" onClick={() => { void load(); }}>Tentar novamente</Button></div></Card> : <>
      {loaded?.source === 'trello' && <TrelloSource id={loaded.id} disabled={busy} onAdd={media => { if (draft.media.some(m => m.checksum === media.checksum)) { setError('Esta mídia já está na composição.'); return; } setDraft(current => ({ ...current, media: [...current.media, media] })); }} />}
      {error && <p role="alert" className="editorial-error">{error}</p>}
      {issues.length > 0 && <div role="alert" className="editorial-error"><p>Alguns arquivos não foram adicionados:</p><ul>{issues.map((issue,i) => <li key={i}>{issue}</li>)}</ul></div>}
      <div className="split-editor publication-editor"><Card><div className="card-heading"><h2>Mídias da publicação</h2><span className="small muted">{draft.media.length} {draft.media.length === 1 ? 'arquivo' : 'arquivos'}</span></div><div className="card-body">
        {draft.media.length === 0 ? <EmptyState icon="media" title="Selecione sua imagem ou vídeo" description="JPEG, PNG ou WebP até 64 MB; MP4 até 512 MB. Os arquivos originais permanecem intactos." /> : <ol className="editor-media-list">{draft.media.map((media,index) => <li key={media.id}><MediaPreview media={media} /><div className="media-file-info"><strong>{index+1}. {media.filename}</strong><span>{media.mimeType} · {fileSize(media.sizeBytes)}{media.width && media.height ? ` · ${media.width} × ${media.height}` : ''}{media.duration !== null ? ' · ' + media.duration.toLocaleString('pt-BR', { maximumFractionDigits: 1 }) + ' s' : ''}</span><div className="page-actions">{draft.format === 'carousel' && <><IconButton label={'Mover para cima: ' + media.filename} disabled={busy || index === 0} onClick={() => { reorder(index,-1); }}>↑</IconButton><IconButton label={'Mover para baixo: ' + media.filename} disabled={busy || index === draft.media.length-1} onClick={() => { reorder(index,1); }}>↓</IconButton></>}<Button variant="ghost" disabled={busy} onClick={() => { void remove(media); }}>Remover mídia {index+1}</Button></div></div></li>)}</ol>}
        <Button variant="secondary" disabled={!native || busy} onClick={() => { void select(); }}>Selecionar mídias</Button>
      </div></Card><Card><div className="card-heading"><h2>Conteúdo da unidade</h2></div><div className="card-body form-stack"><Input label="Título interno" value={draft.title} required maxLength={200} disabled={!native || busy} onChange={e => { setDraft(current => ({ ...current, title: e.target.value })); }} hint="Obrigatório. Usado apenas na organização da biblioteca." /><Select label="Formato editorial" value={draft.format} disabled={!native || busy} onChange={e => { setDraft(current => ({ ...current, format: e.target.value as PublicationFormat })); }}>{Object.entries(formatLabels).map(([value,label]) => <option key={value} value={value}>{label}</option>)}</Select><p className="small muted">Feed: 1 imagem · Carrossel: 2–20 mídias · Reels: 1 vídeo · Stories: 1 imagem ou vídeo. Estes formatos indicam intenção editorial; suporte da API Meta será pesquisado na Fase 07.</p><Textarea label="Legenda" value={draft.caption} rows={10} disabled={!native || busy} onChange={e => { setDraft(current => ({ ...current, caption: e.target.value })); }} hint={`${[...draft.caption].length.toLocaleString('pt-BR')} / 20.000 caracteres. Quebras de linha e emojis são preservados.`} /></div></Card></div>
    </>}
  </>;
}
