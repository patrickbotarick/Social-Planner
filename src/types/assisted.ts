import type { Publication } from './editorial';
import type { CalendarQuery, Schedule } from './calendar';
export interface Preparation {id:string;status:'preparing'|'prepared'|'outdated'|'preparation_failed';fingerprint:string;rootId:string;relativePath:string;publicationVersion:number;scheduleVersion:number;createdAt:string;errorCode:string|null}
export interface ManualConfirmation {id:string;status:'confirmed_manually'|'requires_review'|'superseded';version:number;preparationId:string;plannedDate:string;plannedTime:string;plannedTimezone:string;plannedUtc:string;actualDate:string;actualTime:string;actualTimezone:string;actualUtc:string;platform:'instagram';format:string;reference:string;note:string;confirmedAt:string}
export interface AssistedItem {schedule:Schedule;publication:Publication|null;preparation:Preparation|null;confirmation:ManualConfirmation|null;eligible:boolean;issues:string[];warnings:string[];divergent:boolean}
export interface AssistedCounts {planned:number;notPrepared:number;prepared:number;confirmed:number;outdated:number;divergent:number;failed:number;pendingTransfer:number}
export interface AssistedPage {items:AssistedItem[];total:number;counts:AssistedCounts;exportRoot:string|null;boards:{id:string;name:string}[]}
export interface AssistedQuery {calendar:CalendarQuery;state:string|null;offset:number}
export interface PreparationTarget {scheduleId:string;scheduleVersion:number;publicationVersion:number}
export interface PreparationResult {scheduleId:string;preparationId:string|null;error:{code:string;message:string}|null}
export interface ConfirmationInput {preparationId:string;previousId:string|null;expectedVersion:number|null;actualDate:string;actualTime:string;timezone:string;reference:string;note:string;verifiedByOperator:boolean}
export interface AssistedEvent {id:number;title:string;scheduleId:string;preparationId:string|null;confirmationId:string|null;event:string;createdAt:string}
export interface AssistedHistoryPage {items:AssistedEvent[];total:number}
export const preparationLabels:Record<string,string>={not_prepared:'Não preparada',preparing:'Preparando',prepared:'Preparada localmente',outdated:'Pacote desatualizado',preparation_failed:'Falha na preparação'};
export const confirmationLabels:Record<string,string>={not_confirmed:'Não confirmada',confirmed_manually:'Confirmada manualmente',requires_review:'Requer revisão na Meta',superseded:'Confirmação anterior preservada'};
export const assistedEventLabels:Record<string,string>={preparation_started:'Preparação iniciada',preparation_completed:'Preparação concluída',preparation_failed:'Falha na preparação',package_regenerated:'Pacote regenerado',package_outdated:'Pacote desatualizado',confirmation_recorded:'Confirmação manual registrada',confirmation_revised:'Confirmação revisada',divergence_detected:'Divergência detectada'};
