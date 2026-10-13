import type { ReactNode, RefObject } from 'react';
import { sections } from '../app/navigation';
import type { Section } from '../app/navigation';
import { Icon } from '../components/Icon';
import styles from './AppLayout.module.css';

export function AppLayout({ section, children, mainRef }: { section: Section; children: ReactNode; mainRef: RefObject<HTMLElement | null> }) {
  return <div className={styles.shell}>
    <a className="skip-link" href="#main-content" onClick={(event) => { event.preventDefault(); mainRef.current?.focus(); }}>Ir para o conteúdo</a>
    <aside className={styles.sidebar}>
      <a className={styles.brand} href="#/dashboard" aria-label="GMAD Social Planner — início"><span className={styles.brandMark}>G<span>+</span></span><span><strong>GMAD</strong><small>Social Planner</small></span></a>
      <div className={styles.unit}><span className={styles.unitDot} />Rio Preto<span className={styles.unitCaption}>Unidade</span></div>
      <p className={styles.navCaption}>ESPAÇO EDITORIAL</p>
      <nav aria-label="Navegação principal" className={styles.nav}>
        {sections.map((item) => <a key={item.id} href={`#/${item.id}`} aria-current={item.id === section.id ? 'page' : undefined} aria-label={item.label}><Icon name={item.id} /><span>{item.short}</span>{item.id === section.id && <span className={styles.activeDot} />}</a>)}
      </nav>
      <div className={styles.sidebarFooter}><span className={styles.localMark}>NO SEU COMPUTADOR</span><p>Organização local.<br />Planejamento com cuidado.</p><small>Fundação · Fase 01</small></div>
    </aside>
    <div className={styles.workspace}>
      <header className={styles.header}><span>GMAD Rio Preto <span className={styles.separator}>/</span> <strong>{section.short}</strong></span><span className={styles.phase}>Fase 01 · Fundação</span></header>
      <main id="main-content" tabIndex={-1} ref={mainRef} className={styles.main}>
        <div className={styles.pageHeading}><p className="eyebrow">SOCIAL PLANNER</p><h1>{section.label}</h1><p>{section.description}</p></div>
        {children}
      </main>
      <footer className={styles.footer}><span>GMAD Social Planner</span><span>Ambiente local · Interface inicial</span></footer>
    </div>
  </div>;
}
