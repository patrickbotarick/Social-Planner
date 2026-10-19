import type { MediaAsset, PublicationFormat, PublicationSource } from './editorial';
export type PlanningStatus = 'planned_local' | 'cancelled_local' | 'invalidated_local';
export interface CalendarPublication { id:string; title:string; format:PublicationFormat; source:PublicationSource; version:number; primaryMedia:MediaAsset|null; boardName:string|null }
export interface Schedule { id:string; publication:CalendarPublication; publicationVersion:number; scheduledDate:string; scheduledTime:string; timezone:string; scheduledAtUtc:string; displayDate:string; displayTime:string; status:PlanningStatus; version:number; createdAt:string; updatedAt:string; cancelledAt:string|null; conflict:'exact'|'nearby'|null; mediaUnavailable:boolean }
export interface ScheduleSettings { proximityMinutes:number; timeStepMinutes:number; version:number }
export interface CalendarQuery { year:number; month:number; timezone:string; search:string; format:PublicationFormat|null; source:PublicationSource|null; boardId:string|null; status:PlanningStatus|'all'|null }
export interface AvailableQuery { search:string; format:PublicationFormat|null; source:PublicationSource|null; boardId:string|null; mediaType:'image'|'video'|null; offset:number }
export interface AvailablePage { items:CalendarPublication[]; total:number }
export interface CalendarPage { items:Schedule[]; total:number; emptyDays:number; conflicts:number; formats:Partial<Record<PublicationFormat,number>>; boards:{id:string;name:string}[]; settings:ScheduleSettings }
export interface ScheduleInput { id:string|null; expectedVersion:number|null; publicationId:string; publicationVersion:number; scheduledDate:string; scheduledTime:string; timezone:string }
export interface PlanningDashboard { available:number; plannedThisMonth:number; conflicts:number; upcoming:Schedule[] }
export interface PlanningEvent { id:number; scheduleId:string; publicationId:string; title:string; event:string; previousDate:string|null; previousTime:string|null; previousTimezone:string|null; scheduledDate:string; scheduledTime:string; timezone:string; scheduleVersion:number; publicationVersion:number; createdAt:string }
export interface PlanningHistory { items:PlanningEvent[]; total:number }
export const planningLabels:Record<PlanningStatus,string> = { planned_local:'Planejado localmente',cancelled_local:'Cancelado localmente',invalidated_local:'Revisão necessária' };
export const planningEventLabels:Record<string,string> = { created_local:'Planejamento local criado',rescheduled_local:'Data ou horário alterado',cancelled_local:'Planejamento local cancelado',invalidated_local:'Planejamento invalidado por mudança editorial' };
