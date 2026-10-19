import { invoke } from '@tauri-apps/api/core';
import type { AvailablePage, AvailableQuery, CalendarPage, CalendarQuery, PlanningDashboard, PlanningHistory, ScheduleInput } from '../types/calendar';
export const calendarList = (query:CalendarQuery) => invoke<CalendarPage>('calendar_list',{query});
export const calendarAvailable = (query:AvailableQuery) => invoke<AvailablePage>('calendar_available',{query});
export const saveSchedule = (input:ScheduleInput) => invoke<string>('schedule_save',{input});
export const cancelSchedule = (id:string,expectedVersion:number) => invoke<void>('schedule_cancel',{id,expectedVersion});
export const saveScheduleSettings = (input:{proximityMinutes:number;timeStepMinutes:number;expectedVersion:number}) => invoke<void>('schedule_settings_save',{input});
export const planningDashboard = (timezone:string) => invoke<PlanningDashboard>('planning_dashboard',{timezone});
export const scheduleHistory = (offset:number) => invoke<PlanningHistory>('schedule_history',{offset});
