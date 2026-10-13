import type { Section } from '../app/navigation';
import { Icon } from '../components/Icon';

export function PlannedPage({ section }: { section: Section }) {
  return <section className="panel empty-state"><span className="empty-icon"><Icon name={section.id} /></span><span className="badge">Previsto para a Fase {section.phase}</span><h2>Este espaço está sendo preparado</h2><p>{section.description} A funcionalidade será implementada na Fase {section.phase} do projeto.</p><button className="button" disabled>Disponível em uma próxima fase</button><a className="text-link" href="#/dashboard">Voltar para o Dashboard</a><p className="small muted">Nenhuma importação, alteração editorial ou publicação é executada nesta área.</p></section>;
}
