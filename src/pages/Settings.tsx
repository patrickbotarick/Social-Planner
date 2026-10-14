import { TrelloSettings } from '../features/editorial/TrelloSettings';
import { StorageStatus } from '../components/StorageStatus';
import { PageHeader } from '../components/PageHeader';
import { Tabs } from '../components/ui/Tabs';
import { Input } from '../components/ui/Controls';
import { Badge, Card } from '../components/ui/Presentation';
import { Button } from '../components/ui/Button';
import type { FoundationState } from '../types/foundation';

export function Settings({ state, onRefresh }: { state: FoundationState; onRefresh: () => Promise<void> }) {
  return <>
    <PageHeader title="Configurações" description="Preferências da unidade, armazenamento e conexões externas." />
    <div className="settings-layout"><Tabs label="Seções de configurações" items={[
      { id: 'preferences', label: 'Preferências', content: <div className="settings-grid">
        <Card><div className="card-heading"><h2>Planejamento</h2></div><div className="card-body"><Input label="Fuso de planejamento" value={state.kind === 'ready' ? state.data.settings.timezone : ''} placeholder="Aguardando armazenamento local" readOnly hint="Preferência existente, em modo de consulta. Datas do calendário respeitam este fuso." /><p className="small muted" style={{ marginTop: 20 }}>O planejamento local não representa um agendamento confirmado pela Meta.</p></div></Card>
        <Card><div className="card-heading"><h2>Sobre o aplicativo</h2></div><div className="card-body"><dl className="details"><div><dt>Produto</dt><dd>GMAD Social Planner</dd></div><div><dt>Unidade</dt><dd>GMAD Rio Preto</dd></div><div><dt>Versão</dt><dd>0.1.0</dd></div></dl></div></Card>
      </div> },
      { id: 'storage', label: 'Armazenamento', content: <StorageStatus state={state} onRefresh={onRefresh} /> },
      { id: 'trello', label: 'Trello', content: <TrelloSettings /> },
      { id: 'integrations', label: 'Integrações', content: <Card><div className="card-heading"><h2>Conexões externas</h2><span className="small muted">Outras conexões</span></div>{[
        { name: 'Meta', phase: '07', description: 'Método oficial a validar antes de conectar.' },
      ].map((item) => <div className="integration-row" key={item.name}><div><h3>{item.name} <Badge>Não configurada</Badge></h3><p>{item.description} Disponível na Fase {item.phase}.</p></div><Button disabled variant="secondary">Conectar {item.name}</Button></div>)}</Card> },
    ]} /></div>
  </>;
}
