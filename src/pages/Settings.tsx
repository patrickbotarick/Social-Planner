import { StorageStatus } from '../components/StorageStatus';
import type { FoundationState } from '../types/foundation';

export function Settings({ state, onRefresh }: { state: FoundationState; onRefresh: () => Promise<void> }) {
  return <div className="settings-grid"><StorageStatus state={state} onRefresh={onRefresh} /><section className="panel"><h2>Sobre o aplicativo</h2><dl className="details"><div><dt>Produto</dt><dd>GMAD Social Planner</dd></div><div><dt>Unidade</dt><dd>GMAD Rio Preto</dd></div><div><dt>Versão</dt><dd>0.1.0 · Fundação</dd></div></dl><p className="muted">Preferências em modo de consulta nesta fase. Integrações com Trello e Meta ainda não estão disponíveis.</p><p className="scope-note">O planejamento local não representa agendamento confirmado por uma plataforma.</p></section></div>;
}
