import {useState,useCallback} from 'react';
import {assistedHistory} from '../../services/assisted';
import {useEditorialQuery} from '../../hooks/useEditorialQuery';
import {Button} from '../../components/ui/Button';
import {assistedEventLabels} from '../../types/assisted';
import {editorialDate} from './MediaPreview';
export function AssistedHistory(){
 const [offset,setOffset]=useState(0);const request=useCallback(()=>assistedHistory(offset),[offset]);const {state}=useEditorialQuery(request);
 return <section><h2>Histórico de transferência assistida</h2><p className="small">Preparações e declarações manuais locais. Não é um histórico de publicação verificado pela Meta.</p>{state.kind==='ready'?<><p>{state.data.total} eventos assistidos</p><ol className="planning-upcoming">{state.data.items.map(e=><li key={e.id}><strong>{assistedEventLabels[e.event] ?? e.event} · {e.title}</strong><span className="small">{editorialDate(e.createdAt)}</span><span className="small assisted-path">Plano {e.scheduleId}{e.preparationId?' · pacote '+e.preparationId:''}{e.confirmationId?' · declaração '+e.confirmationId:''}</span></li>)}</ol>{!state.data.items.length && <p>Nenhum evento assistido registrado.</p>}<div className="page-actions"><Button disabled={offset===0} variant="secondary" onClick={()=>{setOffset(n=>Math.max(0,n-30));}}>Eventos assistidos anteriores</Button><span>Página {Math.floor(offset/30)+1}</span><Button disabled={offset+30>=state.data.total} variant="secondary" onClick={()=>{setOffset(n=>n+30);}}>Mais eventos assistidos</Button></div></>:<p role={state.kind==='error'?'alert':'status'}>{state.kind==='error'?state.message:state.kind==='browser'?'Histórico disponível no aplicativo desktop.':'Consultando eventos assistidos…'}</p>}</section>;
}
