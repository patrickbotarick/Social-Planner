import { useCallback, useState } from 'react';
import { useEditorialQuery } from '../../hooks/useEditorialQuery';
import { reviewHistory } from '../../services/review';
import { eventLabels, reasonLabels } from '../../types/review';
import { statusLabels } from '../../types/editorial';
import { Button } from '../../components/ui/Button';
import { Input, Select } from '../../components/ui/Controls';
import { editorialDate } from './MediaPreview';
export function ReviewHistory({ publicationId = null, revision = 0 }: {publicationId?: string | null; revision?: number}) {
  const [search,setSearch]=useState(''); const [event,setEvent]=useState(''); const [offset,setOffset]=useState(0);
  const fetch=useCallback(() => { void revision; return reviewHistory({publicationId,search,event:event || null,offset}); },[publicationId,search,event,offset,revision]);
  const {state,refresh}=useEditorialQuery(fetch);
  return <section aria-label="Histórico editorial" className="review-history"><h2>Histórico editorial</h2><p className="small muted">Decisões locais por versão. Estes registros não representam agendamento ou publicação externa.</p><div className="review-filters"><Input label="Pesquisar histórico" value={search} disabled={state.kind==='browser'} onChange={e=>{setSearch(e.target.value);setOffset(0);}} /><Select label="Evento editorial" value={event} disabled={state.kind==='browser'} onChange={e=>{setEvent(e.target.value);setOffset(0);}}><option value="">Todos os eventos</option>{Object.entries(eventLabels).map(([key,label])=><option key={key} value={key}>{label}</option>)}</Select></div>
    {state.kind!=='ready' ? <p role={state.kind==='error'?'alert':'status'}>{state.kind==='error'?state.message:state.kind==='browser'?'Histórico disponível na janela desktop.':'Consultando histórico…'}{state.kind==='error' && <Button onClick={()=>{void refresh();}}>Tentar novamente</Button>}</p> : <><p className="small">{state.data.total} eventos encontrados</p>{state.data.items.length===0?<p>Nenhum evento editorial encontrado.</p>:<ol className="review-event-list">{state.data.items.map(e=><li key={e.id}><div><strong>{eventLabels[e.event] ?? e.event}</strong> · {e.title}</div><p className="small muted">{editorialDate(e.createdAt)} · {statusLabels[e.previousStatus]} → {statusLabels[e.nextStatus]} · versão {e.reviewedVersion} → {e.resultingVersion}</p>{e.reason && <p>{reasonLabels[e.reason]}</p>}{e.note && <p className="caption-content">{e.note}</p>}</li>)}</ol>}<div className="library-pagination"><Button variant="secondary" disabled={offset===0} onClick={()=>{setOffset(Math.max(0,offset-30));}}>Eventos anteriores</Button><span>Página {Math.floor(offset/30)+1}</span><Button variant="secondary" disabled={offset+30>=state.data.total} onClick={()=>{setOffset(offset+30);}}>Mais eventos</Button></div></>}
  </section>;
}
