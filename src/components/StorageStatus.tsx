import type { FoundationState } from '../types/foundation';

export function StorageStatus({ state, onRefresh }: { state: FoundationState; onRefresh: () => Promise<void> }) {
  return <section className="panel storage" aria-labelledby="storage-title">
    <div className="section-heading"><h2 id="storage-title">Armazenamento local</h2><span className={`badge ${state.kind === 'ready' ? 'badge-success' : ''}`}>{state.kind === 'ready' ? 'Verificado' : state.kind === 'browser' ? 'Prévia web' : state.kind === 'error' ? 'Atenção' : 'Verificando'}</span></div>
    {state.kind === 'loading' && <p role="status">Verificando a conexão com o aplicativo e o banco de dados…</p>}
    {state.kind === 'browser' && <div role="status"><p>Você está na prévia da interface.</p><p className="muted">O SQLite está disponível apenas na janela desktop. Nenhum banco foi conectado nesta prévia.</p></div>}
    {state.kind === 'error' && <p role="alert">{state.message}</p>}
    {state.kind === 'ready' && <><p role="status">Banco conectado. Verificação de integridade concluída.</p><dl className="details"><div><dt>Estrutura do banco</dt><dd>Versão {state.data.schemaVersion}</dd></div><div><dt>Fuso de planejamento</dt><dd>{state.data.settings.timezone}</dd></div><div><dt>Local do banco</dt><dd className="file-path">{state.data.databasePath}</dd></div></dl></>}
    {state.kind !== 'browser' && <button className="button secondary" disabled={state.kind === 'loading'} onClick={() => { void onRefresh(); }}>Verificar novamente</button>}
  </section>;
}
