import { PageHeader, AvailabilityNote } from '../components/PageHeader';
import { Button } from '../components/ui/Button';
import { Input, Select, Textarea } from '../components/ui/Controls';
import { Card, MediaThumbnail } from '../components/ui/Presentation';
export function NewPublication() {
  return <><PageHeader title="Nova publicação" description="Um espaço para os conteúdos próprios da GMAD Rio Preto." actions={<Button disabled>Salvar publicação</Button>} /><div className="split-editor">
    <Card><div className="card-heading"><h2>Mídia</h2></div><div className="card-body"><MediaThumbnail alt="Prévia da nova publicação" emptyLabel="Sua imagem ou vídeo aparecerá aqui" /><div className="form-actions"><Button disabled variant="secondary">Selecionar mídia</Button></div><p className="small muted" style={{ marginTop: 12 }}>Arquivos permanecerão armazenados no computador.</p></div></Card>
    <Card><div className="card-heading"><h2>Detalhes da publicação</h2></div><div className="card-body form-stack"><Input label="Título interno" placeholder="Identifique o conteúdo para a sua organização" disabled /><Select label="Formato" disabled><option>Selecione um formato</option></Select><Textarea label="Legenda" placeholder="Escreva a legenda da publicação" rows={7} disabled hint="A criação manual será disponibilizada junto da biblioteca." /></div></Card>
    </div><AvailabilityNote phase="03">Este formulário é uma estrutura visual. Upload e salvamento ainda não estão disponíveis.</AvailabilityNote></>;
}
