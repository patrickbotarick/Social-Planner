import {useCallback} from 'react';
import {assistedList} from '../../services/assisted';
import {useEditorialQuery} from '../../hooks/useEditorialQuery';
import {todayInTimezone} from '../../utils/calendar';
import {Card} from '../../components/ui/Presentation';
const metrics=[['notPrepared','Pendentes de preparação'],['prepared','Preparadas'],['confirmed','Confirmadas manualmente'],['outdated','Desatualizadas'],['divergent','Com divergência']] as const;
export function AssistedSummary({timezone}:{timezone:string}){
 const request=useCallback(()=>{const today=todayInTimezone(timezone);return assistedList({calendar:{year:today.year,month:today.month+1,timezone,search:'',format:null,source:null,boardId:null,status:null},state:null,offset:0});},[timezone]);
 const {state}=useEditorialQuery(request);
 return <Card aria-label="Transferência assistida"><div className="card-heading"><h2>Transferência assistida</h2><a href="#/assistido">Preparar e acompanhar</a></div><div className="card-body"><p className="small">Mês atual. Confirmações são declarações do operador, sem verificação pela API Meta.</p>{state.kind==='ready'?<dl className="assisted-counts">{metrics.map(([key,label])=><div key={key}><dt>{label}</dt><dd>{state.data.counts[key]}</dd></div>)}</dl>:<p role={state.kind==='error'?'alert':'status'}>{state.kind==='error'?state.message:state.kind==='browser'?'Disponível no aplicativo desktop.':'Consultando transferência…'}</p>}</div></Card>;
}
