import { useEffect, useState } from 'react';
import { Button } from '../../components/ui/Button';
import { Input } from '../../components/ui/Controls';
import { Badge, Card } from '../../components/ui/Presentation';
import { desktopAvailable } from '../../services/desktop';
import { trelloAuthorizationUrl, trelloConnect, trelloDisconnect, trelloError, trelloStatus, trelloTest } from '../../services/trello';
import type { ConnectionStatus } from '../../types/trello';
export function TrelloSettings() {
  const native = desktopAvailable();
  const [status, setStatus] = useState<ConnectionStatus | null>(null);
  const [key, setKey] = useState(''); const [token, setToken] = useState(''); const [authorization, setAuthorization] = useState('');
  const [busy, setBusy] = useState(false); const [message, setMessage] = useState(''); const [failed, setFailed] = useState(false);
  useEffect(() => { let active = true; if (native) void trelloStatus().then(s => { if (active) setStatus(s); }).catch(e => { if (active) { setMessage(trelloError(e)); setFailed(true); } }); return () => { active = false; }; }, [native]);
  const run = async (action: () => Promise<void>) => { setBusy(true); setMessage(''); setFailed(false); try { await action(); } catch (e) { setFailed(true); setMessage(trelloError(e)); } finally { setBusy(false); } };
  return <Card><div className="card-heading"><h2>Trello</h2><Badge>{status ? status.configured ? 'Credencial salva' : 'Desconectado' : native ? 'Consultando cofre…' : 'Disponível no desktop'}</Badge></div><div className="card-body form-stack">
    <p>Conecte uma conta com acesso aos quadros da matriz. O aplicativo consulta o Trello com permissão de leitura.</p>
    {status?.account && <p>Conta verificada: <strong>{status.account.fullName}</strong> · @{status.account.username}</p>}
    {message && <p role={failed ? 'alert' : 'status'} className={failed ? 'editorial-error' : 'editorial-notice'}>{message}</p>}
    {status?.configured && <div className="page-actions"><Button disabled={busy} onClick={() => { void run(async () => { setStatus(await trelloTest()); setMessage('Conexão verificada com o Trello.'); }); }}>Testar conexão</Button><Button variant="secondary" disabled={busy} onClick={() => { void run(async () => { await trelloDisconnect(); setStatus({ configured: false, account: null }); setMessage('Credencial removida deste computador. Para revogar o token, use as configurações da conta Trello.'); }); }}>Desconectar</Button></div>}
    <details open={!status?.configured}><summary>{status?.configured ? 'Renovar autorização' : 'Configurar conexão'}</summary><div className="form-stack">
      <ol><li>Acesse <a href="https://trello.com/apps/admin" target="_blank" rel="noreferrer">o portal de aplicativos do Trello</a>, crie um Power-Up pessoal e gere uma chave na aba Trello Auth.</li><li>Informe a chave abaixo e copie o endereço de autorização. Abra-o no navegador, confira a permissão de leitura e autorize por 30 dias.</li><li>Cole o token recebido neste formulário e conecte. Não compartilhe o token em mensagens.</li></ol>
      <Input label="Chave de API Trello" value={key} autoComplete="off" spellCheck={false} disabled={!native || busy} onChange={e => { setKey(e.target.value.trim()); setAuthorization(''); }} />
      <Button variant="secondary" disabled={!native || busy || !key} onClick={() => { void run(async () => { const url = await trelloAuthorizationUrl(key); setAuthorization(url); await navigator.clipboard.writeText(url); setMessage('Endereço de autorização copiado. Abra-o no navegador.'); }); }}>Copiar endereço de autorização</Button>
      {authorization && <p><a href={authorization} target="_blank" rel="noreferrer">Autorizar leitura no Trello</a></p>}
      <Input label="Token de autorização" type="password" value={token} autoComplete="off" spellCheck={false} disabled={!native || busy} onChange={e => { setToken(e.target.value.trim()); }} hint="Usado uma vez para configurar o cofre do Windows; não é gravado no banco editorial." />
      <Button disabled={!native || busy || !key || !token} onClick={() => { const submittedKey = key; const submittedToken = token; setToken(''); void run(async () => { const s = await trelloConnect(submittedKey, submittedToken); setStatus(s); setKey(''); setAuthorization(''); setMessage('Conexão verificada e credencial protegida no Windows.'); }); }}>{busy ? 'Aguarde…' : 'Conectar Trello'}</Button>
    </div></details>
  </div></Card>;
}
