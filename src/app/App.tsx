import { useEffect, useRef, useState } from 'react';
import { sectionFromHash } from './navigation';
import { useFoundation } from '../hooks/useFoundation';
import { AppLayout } from '../layouts/AppLayout';
import { Dashboard } from '../pages/Dashboard';
import { Settings } from '../pages/Settings';
import { PlannedPage } from '../pages/PlannedPage';

export function App() {
  const [section, setSection] = useState(() => sectionFromHash(window.location.hash));
  const { state, refresh } = useFoundation();
  const main = useRef<HTMLElement>(null);
  useEffect(() => {
    const navigate = () => {
      setSection(sectionFromHash(window.location.hash));
      main.current?.focus();
    };
    window.addEventListener('hashchange', navigate);
    return () => window.removeEventListener('hashchange', navigate);
  }, []);
  useEffect(() => { document.title = `${section.label} · GMAD Social Planner`; }, [section]);

  return <AppLayout section={section} mainRef={main}>
    {section.id === 'dashboard' ? <Dashboard state={state} onRefresh={refresh} />
      : section.id === 'configuracoes' ? <Settings state={state} onRefresh={refresh} />
        : <PlannedPage section={section} />}
  </AppLayout>;
}
