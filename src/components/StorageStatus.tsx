import { useId } from 'react';
import type { FoundationState } from '../types/foundation';
import { Button } from './ui/Button';
import { Badge, Card, Skeleton } from './ui/Presentation';
import { useToast } from './ui/Toast';
import { Icon } from './Icon';

export function StorageStatus({ state, onRefresh, compact = false }: { state: FoundationState; onRefresh: () => Promise<void>; compact?: boolean }) {
  const heading = useId();
  const notify = useToast();
  const copyPath = async () => {
    if (state.kind !== 'ready') return;
    try { await navigator.clipboard.writeText(state.data.databasePath); notify('Caminho do banco copiado.'); }
    catch { notify('Não foi possível copiar o caminho. Selecione o texto e copie manualmente.', 'danger'); }
  };
  return <Card className={compact ? 'storage-compact' : ''} aria-labelledby={heading}>
    <div className="card-heading"><h2 id={heading}>Armazenamento local</h2><Badge tone={state.kind === 'ready' ? 'success' : state.kind === 'error' ? 'danger' : 'neutral'}>{state.kind === 'ready' ? 'Verificado' : state.kind === 'browser' ? 'Prévia web' : state.kind === 'error' ? 'Atenção' : 'Verificando'}</Badge></div>
    <div className="card-body">
      <div className="storage-message">
        {state.kind === 'loading' && <><p role="status">Verificando a conexão com o aplicativo e o banco de dados…</p><div className="skeleton-group"><Skeleton width="85%" /><Skeleton width="60%" /></div></>}
        {state.kind === 'browser' && <div role="status"><p>Você está na prévia da interface.</p><p className="muted">O SQLite está disponível apenas na janela desktop. Nenhum banco foi conectado nesta prévia.</p></div>}
        {state.kind === 'error' && <p role="alert">{state.message}</p>}
        {state.kind === 'ready' && <p role="status">Banco conectado. Verificação de integridade concluída.</p>}
      </div>
      {state.kind === 'ready' && !compact && <dl className="details"><div><dt>Estrutura do banco</dt><dd>Versão {state.data.schemaVersion}</dd></div><div><dt>Fuso de planejamento</dt><dd>{state.data.settings.timezone}</dd></div><div><dt>Local do banco</dt><dd className="file-path">{state.data.databasePath}</dd></div></dl>}
      {!compact && <div className="storage-actions">{state.kind !== 'browser' && <Button variant="secondary" disabled={state.kind === 'loading'} onClick={() => { void onRefresh(); }}>Verificar novamente</Button>}{state.kind === 'ready' && <Button variant="ghost" icon={<Icon name="copy" />} onClick={() => { void copyPath(); }}>Copiar caminho</Button>}</div>}
      {compact && <a href="#/configuracoes" className="small">Ver configurações</a>}
    </div>
  </Card>;
}
