import { useCallback, useEffect, useRef, useState } from 'react';
import type { MouseEvent } from 'react';
import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { sectionFromHash } from './navigation';
import { useFoundation } from '../hooks/useFoundation';
import { AppShell } from '../layouts/AppLayout';
import { Dashboard } from '../pages/Dashboard';
import { Settings } from '../pages/Settings';
import { Library } from '../pages/Library';
import { Imports } from '../pages/Imports';
import { Review } from '../pages/Review';
import { Calendar } from '../pages/Calendar';
import { NewPublication } from '../pages/NewPublication';
import { PublicationDetails } from '../pages/PublicationDetails';
import { History } from '../pages/History';
import { UnsavedDialog } from '../components/ui/Modal';
import { ToastProvider, useToast } from '../components/ui/Toast';
import { editorialError } from '../services/editorial';
import type { LeaveGuard } from '../types/editorial';

export function App() { return <ToastProvider><Application /></ToastProvider>; }
function Application() {
  const [route, setRoute] = useState(() => window.location.hash);
  const section = sectionFromHash(route); const routeRef = useRef(route);
  const { state, refresh } = useFoundation(); const main = useRef<HTMLElement>(null); const notify = useToast();
  const guard = useRef<LeaveGuard | null>(null); const [pending, setPending] = useState<{ run: () => void | Promise<void> } | null>(null); const [leaving, setLeaving] = useState(false);
  const onGuard = useCallback((value: LeaveGuard | null) => { guard.current = value; }, []);
  const attempt = useCallback((run: () => void | Promise<void>) => {
    if (guard.current?.busy) { notify('Aguarde a operação terminar antes de sair.','danger'); return; }
    if (guard.current?.dirty) setPending({ run }); else void run();
  }, [notify]);
  useEffect(() => {
    const navigate = () => {
      const target = window.location.hash;
      if (target === routeRef.current) return;
      if (guard.current?.dirty || guard.current?.busy) {
        window.history.replaceState(null,'',routeRef.current || '#/dashboard');
        attempt(() => { window.location.hash = target; }); return;
      }
      routeRef.current = target; setRoute(target); main.current?.focus();
    };
    window.addEventListener('hashchange',navigate); return () => { window.removeEventListener('hashchange',navigate); };
  }, [attempt]);
  useEffect(() => { document.title = section.label + ' · GMAD Social Planner'; }, [section]);
  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false; let unlisten: (() => void) | undefined;
    void getCurrentWindow().onCloseRequested(event => { if (guard.current?.dirty || guard.current?.busy) { event.preventDefault(); attempt(() => getCurrentWindow().destroy()); } }).then(stop => { if (disposed) stop(); else unlisten = stop; }).catch(() => { notify('Não foi possível ativar a proteção de fechamento. Salve antes de fechar o aplicativo.','danger'); });
    return () => { disposed = true; unlisten?.(); };
  }, [attempt,notify]);
  useEffect(() => { const beforeUnload = (event: BeforeUnloadEvent) => { if (guard.current?.dirty || guard.current?.busy) { event.preventDefault(); event.returnValue = ''; } }; window.addEventListener('beforeunload',beforeUnload); return () => { window.removeEventListener('beforeunload',beforeUnload); }; }, []);
  const discardAndLeave = async () => {
    if (!pending || leaving) return; setLeaving(true);
    try { await guard.current?.discard(); guard.current = null; const run = pending.run; setPending(null); await run(); }
    catch (e) { notify(editorialError(e),'danger'); } finally { setLeaving(false); }
  };
  const saveAndLeave = async () => {
    if (!pending || leaving) return; setLeaving(true);
    try { if (!await guard.current?.save?.()) return; guard.current = null; const run=pending.run; setPending(null); await run(); }
    catch(e) { notify(editorialError(e),'danger'); } finally { setLeaving(false); }
  };
  const intercept = (event: MouseEvent) => {
    const anchor = event.target instanceof Element ? event.target.closest('a[href]') : null;
    const target = anchor?.getAttribute('href');
    if (target?.startsWith('#/') && target !== routeRef.current && (guard.current?.dirty || guard.current?.busy)) { event.preventDefault(); event.stopPropagation(); attempt(() => { window.location.hash = target; }); }
  };
  const publicationRoute = /^#\/biblioteca\/([0-9a-f-]{36})(\/editar)?$/i.exec(route);
  const content = () => {
    if (publicationRoute?.[1]) return publicationRoute[2] ? <NewPublication key={route} id={publicationRoute[1]} onGuard={onGuard} onCancel={() => { attempt(() => { window.location.hash = '#/biblioteca/'+publicationRoute[1]; }); }} onSaved={publication => { window.location.hash = '#/biblioteca/'+publication.id; }} /> : <PublicationDetails key={route} id={publicationRoute[1]} />;
    switch (section.id) {
      case 'dashboard': return <Dashboard state={state} onRefresh={refresh} />;
      case 'biblioteca': return <Library />;
      case 'importacoes': return <Imports onGuard={onGuard} />;
      case 'revisao': return <Review onGuard={onGuard} />;
      case 'calendario': return <Calendar timezone={state.kind === 'ready' ? state.data.settings.timezone : 'America/Sao_Paulo'} onGuard={onGuard} />;
      case 'nova-publicacao': return <NewPublication key={route} onGuard={onGuard} onCancel={() => { attempt(() => { window.location.hash = '#/biblioteca'; }); }} onSaved={publication => { window.location.hash = '#/biblioteca/'+publication.id; }} />;
      case 'historico': return <History />;
      case 'configuracoes': return <Settings state={state} onRefresh={refresh} />;
    }
  };
  return <div onClickCapture={intercept}><AppShell section={section} mainRef={main} state={state}>{content()}</AppShell><UnsavedDialog open={!!pending} busy={leaving} onSave={guard.current?.save ? () => { void saveAndLeave(); } : undefined} onDiscard={() => { void discardAndLeave(); }} onStay={() => { if (!leaving) setPending(null); }} /></div>;
}
