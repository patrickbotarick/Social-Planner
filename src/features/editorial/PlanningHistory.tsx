import { useCallback, useState } from 'react';
import { scheduleHistory } from '../../services/calendar';
import { useEditorialQuery } from '../../hooks/useEditorialQuery';
import { Button } from '../../components/ui/Button';
import { planningEventLabels } from '../../types/calendar';
import { editorialDate } from './MediaPreview';
export function PlanningHistory(){
  const [offset,setOffset]=useState(0);const request=useCallback(()=>scheduleHistory(offset),[offset]);const {state,refresh}=useEditorialQuery(request);
  return <section aria-label="Histórico de planejamento local"><h2>Histórico de planejamento local</h2><p className="small muted">Eventos locais de data e horário, separados de decisões editoriais e publicação externa.</p>
    {state.kind!=='ready'?<p role="status">{state.kind==='error'?<>{state.message} <Button onClick={()=>{void refresh();}}>Tentar novamente</Button></>:state.kind==='browser'?'Disponível no aplicativo desktop.':'Consultando eventos…'}</p>:<>
      <p>{state.data.total} eventos locais</p><ol className="review-event-list">{state.data.items.map(e=><li key={e.id}><strong>{planningEventLabels[e.event] ?? e.event}</strong> · <a href={'#/biblioteca/'+e.publicationId}>{e.title}</a><p>{e.previousDate && <>{e.previousDate} {e.previousTime} ({e.previousTimezone}) → </>}{e.scheduledDate} {e.scheduledTime} ({e.timezone})</p><p className="small muted">{editorialDate(e.createdAt)} · planejamento v{e.scheduleVersion} · publicação v{e.publicationVersion}</p></li>)}</ol>
      <div className="library-pagination"><Button variant="secondary" disabled={offset===0} onClick={()=>{setOffset(v=>Math.max(0,v-30));}}>Eventos locais anteriores</Button><span>Página {offset/30+1}</span><Button variant="secondary" disabled={offset+30>=state.data.total} onClick={()=>{setOffset(v=>v+30);}}>Mais eventos locais</Button></div>
    </>}
  </section>;
}
