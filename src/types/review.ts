import type { EditorialCounts, EditorialStatus, Publication, PublicationFormat, PublicationSource } from './editorial';
export interface ReviewIssue { code: string; severity: 'error' | 'warning'; message: string }
export interface ReviewItem { publication: Publication; boardId: string | null; boardName: string | null; sourceList: string | null; cardUrl: string | null; issues: ReviewIssue[] }
export interface ReviewQuery { search: string; status: EditorialStatus | null; format: PublicationFormat | null; source: PublicationSource | null; boardId: string | null; sourceList: string | null; hasMedia: boolean | null; issuesOnly: boolean; offset: number }
export interface ReviewPage { items: ReviewItem[]; order: string[]; total: number; counts: EditorialCounts; boards: {id: string; name: string}[]; lists: string[] }
export type ReviewAction = 'approve' | 'ignore' | 'reopen' | 'submit';
export interface ReviewTarget { id: string; expectedVersion: number }
export interface ReviewDecision { targets: ReviewTarget[]; action: ReviewAction; reason: string | null; note: string | null }
export interface DecisionResult { id: string; success: boolean; error: {code: string; message: string} | null }
export interface ReviewEvent { id: string; publicationId: string; title: string; event: string; previousStatus: EditorialStatus; nextStatus: EditorialStatus; reviewedVersion: number; resultingVersion: number; reason: string | null; note: string | null; createdAt: string }
export interface HistoryQuery { publicationId: string | null; search: string; event: string | null; offset: number }
export interface HistoryPage { items: ReviewEvent[]; total: number }
export const reasonLabels: Record<string,string> = {product:'Produto não comercializado',campaign:'Campanha não participante',irrelevant:'Conteúdo não relevante',duplicate:'Publicação duplicada editorialmente',other:'Outro'};
export const eventLabels: Record<string,string> = {approved:'Aprovação editorial',ignored:'Publicação ignorada',returned:'Retorno à revisão',submitted:'Envio para revisão',edited:'Conteúdo editado'};
export const initialReviewQuery: ReviewQuery = {search:'',status:'pending_review',format:null,source:null,boardId:null,sourceList:null,hasMedia:null,issuesOnly:false,offset:0};
