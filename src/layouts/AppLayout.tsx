import { useState } from 'react';
import type { ReactNode, RefObject } from 'react';
import type { Section } from '../app/navigation';
import type { FoundationState } from '../types/foundation';
import { Sidebar } from './Sidebar';
import { Topbar } from './Topbar';
import { Modal } from '../components/ui/Modal';
import styles from './AppShell.module.css';

export function AppShell({ section, children, mainRef, state }: { section: Section; children: ReactNode; mainRef: RefObject<HTMLElement | null>; state: FoundationState }) {
  const [helpOpen, setHelpOpen] = useState(false);
  return <div className={styles.shell}>
    <a className="skip-link" href="#main-content" onClick={(event) => { event.preventDefault(); mainRef.current?.focus(); }}>Ir para o conteúdo</a>
    <Sidebar section={section} />
    <div className={styles.workspace}><Topbar section={section} state={state} onHelp={() => { setHelpOpen(true); }} /><main id="main-content" tabIndex={-1} ref={mainRef} className={styles.main}>{children}</main></div>
    <Modal open={helpOpen} title="Sobre o espaço editorial" onClose={() => { setHelpOpen(false); }}><p>Organize o conteúdo da GMAD Rio Preto em um único espaço local.</p><p>As áreas editoriais estão preparadas para receber as próximas funcionalidades. Controles indisponíveis indicam a fase em que serão ativados.</p><p>O armazenamento local já pode ser verificado em Configurações. Um planejamento local nunca representa confirmação de agendamento pela Meta.</p></Modal>
  </div>;
}
