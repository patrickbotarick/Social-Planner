import { PageHeader, AvailabilityNote } from '../components/PageHeader';
import { Button } from '../components/ui/Button';
import { Input } from '../components/ui/Controls';
import { Card, EmptyState } from '../components/ui/Presentation';
import { Icon } from '../components/Icon';
export function Imports() {
  return <><PageHeader title="Importações Trello" description="Receba o conteúdo mensal enviado pela matriz." /><div className="dashboard-columns"><div className="dashboard-main">
    <Card><div className="card-heading"><h2>Importar um quadro</h2></div><div className="card-body"><div className="import-form"><Input label="URL do quadro Trello" type="url" placeholder="https://trello.com/b/…" disabled hint="A conexão e a importação serão disponibilizadas na Fase 04." /><Button disabled icon={<Icon name="importacoes" />}>Importar quadro</Button></div></div></Card>
    <Card><div className="card-heading"><h2>Importações realizadas</h2></div><EmptyState icon="importacoes" title="Nenhum quadro importado nesta versão" description="Os resultados e o andamento das importações aparecerão aqui quando a integração estiver disponível." /></Card>
    </div><Card><div className="card-heading"><h2>Da matriz para sua biblioteca</h2></div><div className="card-body"><ol className="step-list"><li><div><h3>Receba o quadro mensal</h3><p>Use a URL enviada pela matriz GMAD.</p></div></li><li><div><h3>Conecte o Trello</h3><p>A autorização externa será protegida no computador.</p></div></li><li><div><h3>Revise as campanhas</h3><p>Selecione o conteúdo adequado à unidade Rio Preto.</p></div></li></ol></div></Card></div><AvailabilityNote phase="04">Nenhum acesso ao Trello é realizado nesta versão.</AvailabilityNote></>;
}
