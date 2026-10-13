import { useState } from 'react';
import { PageHeader, AvailabilityNote } from '../components/PageHeader';
import { Button, IconButton } from '../components/ui/Button';
import { Card } from '../components/ui/Presentation';
import { Icon } from '../components/Icon';
import { dateKey, monthDays, monthTitle, todayInTimezone } from '../utils/calendar';
export function Calendar({ timezone }: { timezone: string }) {
  const today = todayInTimezone(timezone);
  const [view, setView] = useState(() => ({ year: today.year, month: today.month }));
  const moveMonth = (delta: number) => { setView((current) => { const date = new Date(Date.UTC(current.year, current.month + delta, 1)); return { year: date.getUTCFullYear(), month: date.getUTCMonth() }; }); };
  return <><PageHeader title="Calendário mensal" description="Veja o mês e prepare a distribuição do seu conteúdo." actions={<Button disabled icon={<Icon name="nova-publicacao" />}>Planejar publicação</Button>} />
    <Card aria-label="Calendário editorial"><div className="calendar-toolbar"><div className="month-navigation"><IconButton label="Mês anterior" onClick={() => { moveMonth(-1); }}><Icon name="chevron-left" /></IconButton><IconButton label="Próximo mês" onClick={() => { moveMonth(1); }}><Icon name="chevron-right" /></IconButton><h2 aria-live="polite">{monthTitle(view.year, view.month)}</h2></div><Button variant="secondary" size="small" onClick={() => { setView({ year: today.year, month: today.month }); }}>Hoje</Button></div>
      <div className="calendar-grid" aria-hidden="true">{['Dom', 'Seg', 'Ter', 'Qua', 'Qui', 'Sex', 'Sáb'].map((day) => <div className="weekday" key={day}>{day}</div>)}</div>
      <div className="calendar-grid" role="list" aria-label={'Dias de ' + monthTitle(view.year, view.month)}>{monthDays(view.year, view.month).map((day) => {
        const current = dateKey(day) === dateKey(today);
        return <div role="listitem" className={'calendar-day' + (day.month !== view.month ? ' calendar-day--outside' : '') + (current ? ' calendar-day--today' : '')} key={dateKey(day)}><time dateTime={dateKey(day)} aria-current={current ? 'date' : undefined} aria-label={new Intl.DateTimeFormat('pt-BR', { dateStyle: 'full', timeZone: 'UTC' }).format(new Date(Date.UTC(day.year, day.month, day.day)))}>{day.day}</time></div>;
      })}</div>
    </Card><p className="calendar-caption">Dia atual · {timezone}</p><AvailabilityNote phase="06">A navegação entre meses é visual. Agendamento e alteração de datas estão indisponíveis.</AvailabilityNote></>;
}
