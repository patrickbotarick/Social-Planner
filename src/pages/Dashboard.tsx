import { Icon } from '../components/Icon';
import { getEditorialCounts } from '../services/editorial';
import { desktopAvailable } from '../services/desktop';
import { useEditorialQuery } from '../hooks/useEditorialQuery';
import { PageHeader } from '../components/PageHeader';
import { Button } from '../components/ui/Button';
import { Card } from '../components/ui/Presentation';
import { StorageStatus } from '../components/StorageStatus';
import type { FoundationState } from '../types/foundation';
import { PlanningSummary } from '../features/editorial/PlanningSummary';
import { AssistedSummary } from '../features/editorial/AssistedSummary';

export function Dashboard({ state, onRefresh }: { state: FoundationState; onRefresh: () => Promise<void> }) {
  const counts = useEditorialQuery(getEditorialCounts);
  return <>
    <PageHeader title="Dashboard" description="Uma visão do seu planejamento editorial." actions={<Button disabled={!desktopAvailable()} icon={<Icon name="nova-publicacao" />} onClick={() => { window.location.hash = '#/nova-publicacao'; }}>Nova publicação</Button>} />
    <Card className="editorial-summary" aria-labelledby="summary-title">
      <div className="card-heading"><h2 id="summary-title">Resumo editorial</h2><span className="small muted">Conteúdo da unidade</span></div>
      <dl className="summary-row summary-row--library">{[{ label: 'Total de publicações', key: 'total' }, { label: 'Rascunhos', key: 'drafts' }, { label: 'Pendentes de revisão', key: 'pendingReview' }, { label: 'Aprovadas', key: 'approved' }, { label: 'Ignoradas', key: 'ignored' }].map((item) => <div className="summary-item" key={item.key}><dt>{item.label}</dt><dd aria-label={counts.state.kind === 'ready' ? undefined : 'Contagem indisponível'}>{counts.state.kind === 'ready' ? counts.state.data[item.key as keyof typeof counts.state.data] : '—'}</dd><small>{item.key === 'pendingReview' || item.key === 'approved' || item.key === 'ignored' ? 'Revisão editorial local' : 'Biblioteca local'}</small></div>)}</dl>
      <p className="summary-note" role={counts.state.kind === 'error' ? 'alert' : 'status'}>{counts.state.kind === 'error' ? counts.state.message : counts.state.kind === 'browser' ? 'Os indicadores reais estão disponíveis na janela desktop.' : counts.state.kind === 'loading' ? 'Consultando indicadores…' : 'Contagens reais do banco local. Publicações excluídas não são contabilizadas.'}</p>
    </Card>
    <div className="dashboard-columns">
      <div className="dashboard-main">
        <PlanningSummary timezone={state.kind==='ready'?state.data.settings.timezone:'America/Sao_Paulo'}/>
        <AssistedSummary timezone={state.kind==='ready'?state.data.settings.timezone:'America/Sao_Paulo'}/>
      </div>
      <div className="dashboard-aside"><Card aria-labelledby="shortcuts-title"><div className="card-heading"><h2 id="shortcuts-title">Seu fluxo de trabalho</h2></div>
        {[
          { id: 'importacoes', title: 'Receber da matriz', text: 'Traga cartões e mídias do quadro Trello.', action: 'Importar do Trello', phase: '04' },
          { id: 'revisao', title: 'Selecionar conteúdo', text: 'Revise as campanhas para Rio Preto.', action: 'Revisar publicações', phase: '05' },
          { id: 'nova-publicacao', title: 'Criar na unidade', text: 'Prepare os conteúdos próprios da loja.', action: 'Criar publicação', phase: '03' },
        ].map((item) => <a className="shortcut" href={'#/' + item.id} key={item.id} aria-label={'Abrir área: ' + item.action}><h3><Icon name={item.id} />{item.title}</h3><p>{item.text}</p><span className="small muted">{item.id === 'nova-publicacao' ? 'Cadastrar publicação local' : item.id === 'importacoes' ? 'Consultar quadro e selecionar cartões' : 'Abrir fila editorial'} <span aria-hidden="true">→</span></span></a>)}
      </Card>{state.kind !== 'ready' && <StorageStatus state={state} onRefresh={onRefresh} compact />}</div>
    </div>
  </>;
}
