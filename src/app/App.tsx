import { useEffect, useRef, useState } from 'react';
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
import { History } from '../pages/History';
import { ToastProvider } from '../components/ui/Toast';

export function App() {
  const [section, setSection] = useState(() => sectionFromHash(window.location.hash));
  const { state, refresh } = useFoundation();
  const main = useRef<HTMLElement>(null);
  useEffect(() => {
    const navigate = () => { setSection(sectionFromHash(window.location.hash)); main.current?.focus(); };
    window.addEventListener('hashchange', navigate);
    return () => window.removeEventListener('hashchange', navigate);
  }, []);
  useEffect(() => { document.title = section.label + ' · GMAD Social Planner'; }, [section]);
  const content = () => {
    switch (section.id) {
      case 'dashboard': return <Dashboard state={state} onRefresh={refresh} />;
      case 'biblioteca': return <Library />;
      case 'importacoes': return <Imports />;
      case 'revisao': return <Review />;
      case 'calendario': return <Calendar timezone={state.kind === 'ready' ? state.data.settings.timezone : 'America/Sao_Paulo'} />;
      case 'nova-publicacao': return <NewPublication />;
      case 'historico': return <History />;
      case 'configuracoes': return <Settings state={state} onRefresh={refresh} />;
    }
  };
  return <ToastProvider><AppShell section={section} mainRef={main} state={state}>{content()}</AppShell></ToastProvider>;
}
