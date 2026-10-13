import { PageHeader, AvailabilityNote } from '../components/PageHeader';
import { SearchField, Select } from '../components/ui/Controls';
import { Card, EmptyState, Toolbar } from '../components/ui/Presentation';
export function History() {
  return <><PageHeader title="Histórico" description="Eventos e confirmações de publicação, em ordem cronológica." /><Toolbar label="Filtros do histórico"><SearchField label="Pesquisar histórico" placeholder="Pesquisar publicação" disabled /><Select label="Tipo de evento" hideLabel disabled><option>Todos os eventos</option></Select><Select label="Período" hideLabel disabled><option>Todos os períodos</option></Select></Toolbar>
    <Card><table className="history-table" aria-label="Eventos de publicação"><thead><tr><th scope="col">Data e hora</th><th scope="col">Publicação</th><th scope="col">Evento</th><th scope="col">Estado</th></tr></thead><tbody><tr><td colSpan={4}><EmptyState icon="historico" title="O histórico será construído com suas publicações" description="Envios, confirmações e falhas serão registrados aqui quando o acompanhamento estiver disponível. Nenhum evento de plataforma é simulado." /></td></tr></tbody></table></Card><AvailabilityNote phase="08">Acompanhamento e filtros ainda indisponíveis.</AvailabilityNote></>;
}
