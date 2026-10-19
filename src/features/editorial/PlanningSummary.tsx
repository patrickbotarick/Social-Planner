import { useCallback } from 'react';
import { planningDashboard } from '../../services/calendar';
import { useEditorialQuery } from '../../hooks/useEditorialQuery';
import { Card } from '../../components/ui/Presentation';
import { planningLabels } from '../../types/calendar';
export function PlanningSummary({timezone}:{timezone:string}) {
  const request=useCallback(()=>planningDashboard(timezone),[timezone]);
  const {state}=useEditorialQuery(request);
  return <Card aria-label="Planejamento local"><div className="card-heading"><h2>Planejamento local</h2><a href="#/calendario">Abrir calendário</a></div><div className="card-body">
    <p className="small muted">Intenções neste computador. Sem envio ou confirmação pela Meta.</p>
    {state.kind!=='ready'?<p role="status">{state.kind==='error'?state.message:state.kind==='browser'?'Disponível no aplicativo desktop.':'Consultando planejamento…'}</p>:<>
      <dl className="planning-metrics"><div><dt>Aprovadas disponíveis</dt><dd>{state.data.available}</dd></div><div><dt>Planejadas no mês</dt><dd>{state.data.plannedThisMonth}</dd></div><div><dt>Posts com conflitos no mês</dt><dd>{state.data.conflicts}</dd></div></dl>
      <h3>Próximas publicações locais</h3>{state.data.upcoming.length===0?<p>Nenhum planejamento local futuro.</p>:<ol className="planning-upcoming">{state.data.upcoming.map(s=><li key={s.id}><time dateTime={s.scheduledAtUtc}>{new Intl.DateTimeFormat('pt-BR',{dateStyle:'short',timeStyle:'short',timeZone:timezone,hourCycle:'h23'}).format(new Date(s.scheduledAtUtc))}</time><a href={'#/biblioteca/'+s.publication.id}>{s.publication.title}</a><span className="small">{s.mediaUnavailable?'Mídia indisponível — requer correção':planningLabels[s.status]}{s.conflict?' · Conflito de horário':''}</span></li>)}</ol>}
    </>}
  </div></Card>;
}
