import { PageHeader, AvailabilityNote } from '../components/PageHeader';
import { Button, IconButton } from '../components/ui/Button';
import { SearchField, Select } from '../components/ui/Controls';
import { Card, EmptyState, Toolbar } from '../components/ui/Presentation';
import { Icon } from '../components/Icon';
export function Library() {
  return <><PageHeader title="Biblioteca editorial" description="Seu acervo de campanhas, textos e mídias." actions={<Button disabled icon={<Icon name="nova-publicacao" />}>Nova publicação</Button>} />
    <Toolbar label="Filtros da biblioteca"><SearchField label="Pesquisar publicações" placeholder="Pesquisar título ou campanha" disabled /><Select label="Estado editorial" hideLabel disabled><option>Todos os estados</option></Select><Select label="Origem" hideLabel disabled><option>Todas as origens</option></Select><div className="toolbar-end"><IconButton label="Visualização em grade" disabled aria-pressed="true"><Icon name="dashboard" /></IconButton><IconButton label="Visualização em lista" disabled aria-pressed="false"><Icon name="list" /></IconButton></div></Toolbar>
    <Card className="library-frame" aria-label="Publicações da biblioteca"><p className="library-column-note">A biblioteca receberá as publicações criadas na unidade e os cartões importados da matriz.</p><div className="publication-grid"><EmptyState icon="biblioteca" title="Um acervo pronto para receber conteúdo" description="A criação e a organização de publicações serão disponibilizadas na Fase 03. Pesquisa e filtros serão ativados junto da biblioteca." action={<Button variant="secondary" disabled>Criar primeira publicação</Button>} /></div></Card>
    <AvailabilityNote phase="03">Pesquisa, filtros e criação estão indisponíveis nesta versão.</AvailabilityNote></>;
}
