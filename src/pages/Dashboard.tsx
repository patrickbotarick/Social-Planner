import { Icon } from '../components/Icon';
import { PageHeader } from '../components/PageHeader';
import { Button } from '../components/ui/Button';
import { Card, EmptyState } from '../components/ui/Presentation';
import { StorageStatus } from '../components/StorageStatus';
import type { FoundationState } from '../types/foundation';

export function Dashboard({ state, onRefresh }: { state: FoundationState; onRefresh: () => Promise<void> }) {
  return <>
    <PageHeader title="Dashboard" description="Uma visão do seu planejamento editorial." actions={<Button disabled icon={<Icon name="nova-publicacao" />}>Nova publicação</Button>} />
    <Card className="editorial-summary" aria-labelledby="summary-title">
      <div className="card-heading"><h2 id="summary-title">Resumo editorial</h2><span className="small muted">Conteúdo da unidade</span></div>
      <dl className="summary-row">{[{ label: 'Pendentes de revisão', phase: '05' }, { label: 'Aprovadas', phase: '05' }, { label: 'Planejadas', phase: '06' }].map((item) => <div className="summary-item" key={item.label}><dt>{item.label}</dt><dd aria-label="Contagem indisponível">—</dd><small>Disponível na Fase {item.phase}</small></div>)}</dl>
      <p className="summary-note">Os contadores serão preenchidos quando a biblioteca editorial estiver disponível. Nenhuma contagem é simulada.</p>
    </Card>
    <div className="dashboard-columns">
      <div className="dashboard-main">
        <Card aria-labelledby="upcoming-title"><div className="card-heading"><h2 id="upcoming-title">Próximas publicações</h2><a href="#/calendario" className="small">Abrir calendário</a></div>
          <div className="list-columns" aria-hidden="true"><span>Data</span><span>Publicação</span><span>Estado</span></div>
          <EmptyState icon="calendario" title="Seu planejamento começa aqui" description="As publicações planejadas aparecerão nesta lista quando o calendário editorial estiver disponível." />
        </Card>
      </div>
      <div className="dashboard-aside"><Card aria-labelledby="shortcuts-title"><div className="card-heading"><h2 id="shortcuts-title">Seu fluxo de trabalho</h2></div>
        {[
          { id: 'importacoes', title: 'Receber da matriz', text: 'Traga cartões e mídias do quadro Trello.', action: 'Importar do Trello', phase: '04' },
          { id: 'revisao', title: 'Selecionar conteúdo', text: 'Revise as campanhas para Rio Preto.', action: 'Revisar publicações', phase: '05' },
          { id: 'nova-publicacao', title: 'Criar na unidade', text: 'Prepare os conteúdos próprios da loja.', action: 'Criar publicação', phase: '03' },
        ].map((item) => <a className="shortcut" href={'#/' + item.id} key={item.id} aria-label={'Abrir área: ' + item.action}><h3><Icon name={item.id} />{item.title}</h3><p>{item.text}</p><span className="small muted">Funcionalidade prevista na Fase {item.phase} <span aria-hidden="true">→</span></span></a>)}
      </Card>{state.kind !== 'ready' && <StorageStatus state={state} onRefresh={onRefresh} compact />}</div>
    </div>
  </>;
}
