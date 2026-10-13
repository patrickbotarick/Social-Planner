import type { Section } from '../app/navigation';
import type { FoundationState } from '../types/foundation';
import { IconButton } from '../components/ui/Button';
import { Tooltip } from '../components/ui/Tooltip';
import { Icon } from '../components/Icon';
import styles from './AppShell.module.css';
export function Topbar({ section, state, onHelp }: { section: Section; state: FoundationState; onHelp: () => void }) {
  const label = state.kind === 'ready' ? 'Armazenamento conectado' : state.kind === 'browser' ? 'Prévia web' : state.kind === 'error' ? 'Verifique o armazenamento' : 'Verificando armazenamento';
  return <header className={styles.topbar}><div className={styles.breadcrumb}><span>GMAD Rio Preto</span><span aria-hidden="true">/</span><strong>{section.short}</strong></div><div className={styles.topbarEnd}><span className={`${styles.local} ${state.kind === 'ready' ? styles.localReady : ''}`}>{label}</span><Tooltip text="Sobre o espaço editorial"><IconButton label="Ajuda do aplicativo" onClick={onHelp}><Icon name="help" /></IconButton></Tooltip></div></header>;
}
