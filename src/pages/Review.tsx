import { PageHeader, AvailabilityNote } from '../components/PageHeader';
import { Button, IconButton } from '../components/ui/Button';
import { Textarea } from '../components/ui/Controls';
import { Card, MediaThumbnail } from '../components/ui/Presentation';
import { Icon } from '../components/Icon';
export function Review() {
  return <><PageHeader title="Revisão de publicações" description="Selecione o conteúdo que faz sentido para a sua unidade." actions={<div className="review-navigation"><IconButton label="Publicação anterior" disabled><Icon name="chevron-left" /></IconButton><span>Nenhuma publicação em revisão</span><IconButton label="Próxima publicação" disabled><Icon name="chevron-right" /></IconButton></div>} />
    <div className="split-editor"><Card><div className="card-heading"><h2>Mídia e conteúdo de origem</h2></div><div className="card-body"><MediaThumbnail alt="Prévia da publicação em revisão" emptyLabel="Selecione uma publicação para visualizar" /><dl className="review-source"><div><dt>Título de origem</dt><dd>O título do cartão aparecerá aqui.</dd></div><div><dt>Campanha e produto</dt><dd>Informações preservadas da matriz.</dd></div><div><dt>Descrição de origem</dt><dd>O texto original será apresentado para comparação.</dd></div></dl></div></Card>
    <Card><div className="card-heading"><h2>Conteúdo da unidade</h2></div><div className="card-body"><Textarea label="Legenda" placeholder="A legenda editável será exibida aqui" disabled rows={8} hint="A edição não alterará o texto original recebido da matriz." /><div className="form-actions"><Button disabled icon={<Icon name="check" />}>Aprovar</Button><Button disabled variant="secondary">Ignorar</Button><Button disabled variant="secondary">Salvar alterações</Button></div><AvailabilityNote phase="05">Seleção, edição e aprovação serão ativadas com a revisão.</AvailabilityNote></div></Card></div></>;
}
