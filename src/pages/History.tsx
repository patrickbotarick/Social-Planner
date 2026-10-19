import { PageHeader } from '../components/PageHeader';
import { Card } from '../components/ui/Presentation';
import { ReviewHistory } from '../features/editorial/ReviewHistory';
import { PlanningHistory } from '../features/editorial/PlanningHistory';
import { Tabs } from '../components/ui/Tabs';
import { AssistedHistory } from '../features/editorial/AssistedHistory';
export function History() { return <><PageHeader title="Histórico" description="Decisões, planejamento e declarações manuais registrados neste computador."/><Card><div className="card-body"><Tabs label="Tipo de histórico" items={[{id:'editorial',label:'Editorial',content:<ReviewHistory/>},{id:'planning',label:'Planejamento local',content:<PlanningHistory/>},{id:'assisted',label:'Transferência assistida',content:<AssistedHistory/>}]}/></div></Card></>; }
