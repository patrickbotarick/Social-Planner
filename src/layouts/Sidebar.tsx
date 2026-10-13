import { sections } from '../app/navigation';
import type { Section } from '../app/navigation';
import { Icon } from '../components/Icon';
import styles from './AppShell.module.css';
const groups = [
  { label: 'Planejamento', ids: ['dashboard', 'biblioteca', 'calendario'] },
  { label: 'Conteúdo', ids: ['importacoes', 'revisao', 'nova-publicacao'] },
  { label: 'Acompanhamento', ids: ['historico'] },
];
export function Sidebar({ section }: { section: Section }) {
  const link = (item: Section) => <a className={styles.navLink} key={item.id} href={`#/${item.id}`} aria-current={section.id === item.id ? 'page' : undefined} aria-label={item.label}><Icon name={item.id} /><span>{item.short}</span></a>;
  return <aside className={styles.sidebar}>
    <a href="#/dashboard" className={styles.brand} aria-label="GMAD Social Planner — início"><img className={styles.brandMark} src="/brand.svg" width="32" height="32" alt="" /><span><strong>GMAD</strong><small>Social Planner</small></span></a>
    <div className={styles.unit}>Rio Preto<span>Unidade</span></div>
    <nav aria-label="Navegação principal">{groups.map((group) => <div className={styles.group} key={group.label}><h2>{group.label}</h2>{group.ids.map((id) => sections.find((item) => item.id === id)).filter((item): item is Section => Boolean(item)).map(link)}</div>)}<div className={styles.sidebarBottom}>{link(sections[7])}<p>Seu conteúdo, neste computador.</p></div></nav>
  </aside>;
}
