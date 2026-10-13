import { Icon } from '../components/Icon';
import { StorageStatus } from '../components/StorageStatus';
import type { FoundationState } from '../types/foundation';

export function Dashboard({ state, onRefresh }: { state: FoundationState; onRefresh: () => Promise<void> }) {
  return <>
    <section className="welcome-panel"><div><p className="eyebrow">UM LUGAR PARA CADA IDEIA</p><h2>O próximo mês começa<br />com um bom planejamento.</h2><p>A base do seu espaço editorial está sendo preparada.<br />Explore as áreas que vão acompanhar sua rotina.</p><a className="button light" href="#/biblioteca">Conhecer a biblioteca <span aria-hidden="true">→</span></a></div><div className="welcome-art" aria-hidden="true"><div className="paper"><span>PLANEJAR</span><div className="paper-grid">{Array.from({ length: 12 }, (_, i) => <i key={i} />)}</div><b>Uma ideia de cada vez.</b></div><div className="art-accent">+</div></div></section>
    <div className="section-heading section-spaced"><h2>Seu fluxo editorial</h2><span className="muted small">Construído em etapas</span></div>
    <div className="flow-grid">{[
      { icon: 'importacoes', title: 'Receber', description: 'Campanhas e materiais da matriz.', phase: '04', href: 'importacoes' },
      { icon: 'revisao', title: 'Selecionar', description: 'Conteúdo alinhado à sua unidade.', phase: '05', href: 'revisao' },
      { icon: 'calendario', title: 'Planejar', description: 'Cada publicação no seu momento.', phase: '06', href: 'calendario' },
    ].map((item) => <a className="flow-card" href={`#/${item.href}`} key={item.title}><div className="flow-top"><span className="icon-box"><Icon name={item.icon} /></span><span className="badge">Fase {item.phase}</span></div><h3>{item.title}</h3><p>{item.description}</p><span className="card-link">Ver área <span aria-hidden="true">↗</span></span></a>)}</div>
    <div className="bottom-grid"><StorageStatus state={state} onRefresh={onRefresh} /><section className="panel scope-panel"><span className="eyebrow">NESTA VERSÃO</span><h2>Primeiro, uma base sólida.</h2><p>Navegação, estrutura local e documentação para evoluir com segurança.</p><ul><li>Áreas editoriais organizadas</li><li>Dados sob controle da unidade</li><li>Integrações previstas no roadmap</li></ul><p className="scope-note">Importação, revisão e agendamento serão disponibilizados nas próximas fases.</p></section></div>
  </>;
}
