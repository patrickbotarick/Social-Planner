import { TrelloSettings } from '../features/editorial/TrelloSettings';
import { StorageStatus } from '../components/StorageStatus';
import { PageHeader } from '../components/PageHeader';
import { Tabs } from '../components/ui/Tabs';
import { Input } from '../components/ui/Controls';
import { Card } from '../components/ui/Presentation';
import type { FoundationState } from '../types/foundation';
import type {LeaveGuard} from '../types/editorial';
import {BackupSettings} from '../features/editorial/BackupSettings';
import {version} from '../../package.json';

export function Settings({ state, onRefresh,onGuard }: { state: FoundationState; onRefresh: () => Promise<void>;onGuard?:(guard:LeaveGuard|null)=>void }) {
  return <>
    <PageHeader title="Configurações" description="Preferências da unidade, armazenamento e conexões externas." />
    <div className="settings-layout"><Tabs label="Seções de configurações" items={[
      { id: 'preferences', label: 'Preferências', content: <div className="settings-grid">
        <Card><div className="card-heading"><h2>Planejamento</h2></div><div className="card-body"><Input label="Fuso de planejamento" value={state.kind === 'ready' ? state.data.settings.timezone : ''} placeholder="Aguardando armazenamento local" readOnly hint="Preferência existente, em modo de consulta. Datas do calendário respeitam este fuso." /><p className="small muted" style={{ marginTop: 20 }}>O planejamento local não representa um agendamento confirmado pela Meta.</p></div></Card>
        <Card><div className="card-heading"><h2>Sobre o aplicativo</h2></div><div className="card-body"><dl className="details"><div><dt>Produto</dt><dd>GMAD Social Planner</dd></div><div><dt>Unidade</dt><dd>GMAD Rio Preto</dd></div><div><dt>Versão candidata</dt><dd>{version}</dd></div></dl></div></Card>
      </div> },
      { id: 'storage', label: 'Armazenamento', content: <StorageStatus state={state} onRefresh={onRefresh} /> },
      { id: 'backup', label: 'Backup', content: <BackupSettings onRefresh={onRefresh} {...(onGuard?{onGuard}:{})} /> },
      { id: 'trello', label: 'Trello', content: <TrelloSettings /> },
      { id: 'integrations', label: 'Integrações', content: <Card><div className="card-heading"><h2>Conexões externas</h2></div><div className="card-body"><h3>Meta — fluxo assistido</h3><p>Prepare os materiais e programe manualmente no Business Suite. O GMAD não conecta uma conta Meta por API nem envia publicações.</p><a href="#/assistido">Preparar para Meta</a></div></Card> },
    ]} /></div>
  </>;
}
