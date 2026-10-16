import { PageHeader } from '../components/PageHeader';
import { Card } from '../components/ui/Presentation';
import { ReviewHistory } from '../features/editorial/ReviewHistory';
export function History() { return <><PageHeader title="Histórico" description="Decisões e alterações editoriais registradas neste computador."/><Card><div className="card-body"><ReviewHistory/></div></Card></>; }
