import { useCallback, useEffect, useState } from 'react';
import { Button } from '../../components/ui/Button';
import { Card, Badge } from '../../components/ui/Presentation';
import { Select, Textarea } from '../../components/ui/Controls';
import { PageHeader } from '../../components/PageHeader';
import { useToast } from '../../components/ui/Toast';
import { useEditorialQuery } from '../../hooks/useEditorialQuery';
import { reviewDecide, reviewGet } from '../../services/review';
import { editorialError } from '../../services/editorial';
import { formatLabels, sourceLabels, statusLabels } from '../../types/editorial';
import type { LeaveGuard } from '../../types/editorial';
import type { ReviewAction } from '../../types/review';
import { reasonLabels } from '../../types/review';
import { MediaPreview, fileSize } from './MediaPreview';
import { CaptionText } from './CaptionText';
import { PublicationEditor } from './PublicationEditor';
import { TrelloSource } from './TrelloSource';
import { ReviewHistory } from './ReviewHistory';

interface Props { id:string; position:number; total:number; onMove:(delta:number)=>void; onBack:()=>void; onChanged:()=>void; onGuard:(guard:LeaveGuard|null)=>void; onAttempt:(run:()=>void)=>void }
export function ReviewIndividual({id,position,total,onMove,onBack,onChanged,onGuard,onAttempt}:Props) {
  const [revision,setRevision]=useState(0); const request=useCallback(()=>{void revision;return reviewGet(id);},[id,revision]); const {state,refresh}=useEditorialQuery(request);
  const [editing,setEditing]=useState(false); const [active,setActive]=useState(0); const [busy,setBusy]=useState(false); const [reason,setReason]=useState(''); const [note,setNote]=useState(''); const [failedPlayback,setFailedPlayback]=useState(false); const [error,setError]=useState(''); const notify=useToast();
  useEffect(()=>{if(editing)return;onGuard({dirty:false,busy,discard:()=>Promise.resolve()});return()=>{onGuard(null);};},[editing,busy,onGuard]);
  useEffect(()=>{const key=(e:KeyboardEvent)=>{if(e.repeat || e.altKey || e.ctrlKey || e.metaKey || e.shiftKey || e.defaultPrevented || e.target instanceof Element && e.target.closest('input,textarea,select,button,a,video,[contenteditable],dialog'))return;if(e.key==='ArrowRight' && position<total){e.preventDefault();onMove(1);}if(e.key==='ArrowLeft' && position>1){e.preventDefault();onMove(-1);}};window.addEventListener('keydown',key);return()=>{window.removeEventListener('keydown',key);};},[onMove,position,total]);
  const decide=async(action:ReviewAction)=>{
    if(state.kind!=='ready' || busy)return;setBusy(true);setError('');
    try{const [result]=await reviewDecide({targets:[{id,expectedVersion:state.data.publication.version}],action,reason:action==='ignore' ? reason || null:null,note:action==='ignore' ? note || null:null});if(!result?.success){setError(result?.error ? editorialError(result.error):'Não foi possível registrar a decisão.');return;}notify(action==='approve'?'Publicação aprovada.':action==='ignore'?'Publicação ignorada.':'Publicação enviada à revisão.');setRevision(r=>r+1);onChanged();}
    catch(e){setError(editorialError(e));}finally{setBusy(false);}
  };
  const navigation=<nav className="review-navigation" aria-label="Navegação da revisão"><Button variant="secondary" onClick={onBack} disabled={busy}>Voltar à fila</Button><Button variant="secondary" disabled={busy || position<=1} onClick={()=>{onMove(-1);}}>Publicação anterior</Button><span>{position} de {total} na seleção aberta</span><Button variant="secondary" disabled={busy || position>=total} onClick={()=>{onMove(1);}}>Próxima publicação</Button></nav>;
  if(editing)return <>{navigation}<PublicationEditor id={id} onGuard={onGuard} onCancel={()=>{onAttempt(()=>{setEditing(false);});}} onSaved={()=>{setEditing(false);setFailedPlayback(false);setRevision(r=>r+1);onChanged();}} /></>;
  if(state.kind!=='ready')return <><PageHeader title="Revisão de publicações" description="Análise individual do conteúdo local." />{navigation}<p role={state.kind==='error'?'alert':'status'}>{state.kind==='error'?state.message:'Verificando publicação e mídias…'}</p>{state.kind==='error' && <Button onClick={()=>{void refresh();}}>Tentar novamente</Button>}</>;
  const item=state.data; const p=item.publication; const media=p.media[active] ?? p.media[0]; const pending=['pending_review','editing'].includes(p.editorialStatus); const blocked=item.issues.some(i=>i.severity==='error') || failedPlayback;
  return <><PageHeader title={p.title} description={sourceLabels[p.source]+' · '+formatLabels[p.format]} actions={<Badge>{statusLabels[p.editorialStatus]}</Badge>} />{navigation}<p className="small muted">← / → navegam fora de campos e controles. Aprovação é uma decisão editorial local.</p>{error && <p className="editorial-error" role="alert">{error}</p>}
    <div className="review-focus"><Card><div className="card-heading"><h2>Mídias</h2><span>{p.media.length ? Math.min(active+1,p.media.length):0} / {p.media.length}</span></div><div className="card-body">{media ? <><MediaPreview key={media.id} media={media} onUnavailable={()=>{setFailedPlayback(true);}} /><p className="media-file-info"><strong>{media.filename}</strong><span>{media.mimeType} · {fileSize(media.sizeBytes)}</span></p></>:<p>Nenhuma mídia associada.</p>}<div role="group" aria-label="Mídias da revisão" className="media-gallery-navigation">{p.media.map((m,i)=><Button key={m.id} size="small" variant={active===i?'primary':'secondary'} aria-pressed={active===i} aria-label={`Mídia ${i+1}: ${m.filename}`} onClick={()=>{setActive(i);}}>{i+1}</Button>)}</div></div></Card>
      <Card><div className="card-heading"><h2>Conteúdo da unidade</h2><Button variant="secondary" disabled={busy} onClick={()=>{setEditing(true);}}>Editar</Button></div><div className="card-body form-stack"><dl className="review-metadata"><div><dt>Origem</dt><dd>{sourceLabels[p.source]}</dd></div><div><dt>Quadro</dt><dd>{item.boardName ?? 'Sem quadro Trello'}</dd></div><div><dt>Lista de origem</dt><dd>{item.sourceList ?? '—'}</dd></div><div><dt>Versão</dt><dd>{p.version}</dd></div></dl>{item.cardUrl && <a href={item.cardUrl} target="_blank" rel="noreferrer">Cartão original ↗</a>}<div className="caption-content">{p.source==='trello'?<CaptionText text={p.caption}/>:p.caption || 'Sem legenda.'}</div>
        {item.issues.length>0 && <ul className="review-issues" aria-label="Compatibilidade editorial">{item.issues.map((i,n)=><li key={n} className={'review-issue review-issue--'+i.severity}><strong>{i.severity==='error'?'Impede aprovação':'Atenção'}</strong>: {i.message}</li>)}</ul>}{failedPlayback && <p role="alert" className="editorial-error">O WebView2 não conseguiu exibir esta mídia. Confira o arquivo e o codec no editor antes de aprovar. Isso não indica, por si só, falha no download.</p>}
        <div className="page-actions">{pending && <><Button disabled={busy || blocked} onClick={()=>{void decide('approve');}}>Aprovar</Button><Button variant="secondary" disabled={busy} onClick={()=>{void decide('ignore');}}>Ignorar</Button></>}{['approved','ignored'].includes(p.editorialStatus) && <Button disabled={busy} onClick={()=>{void decide('reopen');}}>Retornar à revisão</Button>}{p.editorialStatus==='draft' && <Button disabled={busy} onClick={()=>{void decide('submit');}}>Enviar para revisão</Button>}</div>
        {pending && <details><summary>Motivo e observação ao ignorar (opcionais)</summary><div className="form-stack"><Select label="Motivo de ignorar" value={reason} disabled={busy} onChange={e=>{setReason(e.target.value);}}><option value="">Sem motivo informado</option>{Object.entries(reasonLabels).map(([k,label])=><option key={k} value={k}>{label}</option>)}</Select><Textarea label="Observação editorial" value={note} maxLength={2000} rows={3} disabled={busy} onChange={e=>{setNote(e.target.value);}} /></div></details>}
      </div></Card></div>{p.source==='trello' && <TrelloSource id={id}/>}<Card><div className="card-body"><ReviewHistory publicationId={id} revision={revision}/></div></Card>
  </>;
}
