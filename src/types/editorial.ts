export type PublicationFormat = 'feed' | 'carousel' | 'reels' | 'stories';
export type EditorialStatus = 'draft' | 'pending_review' | 'editing' | 'approved' | 'ignored' | 'ready_to_schedule';
export type PublicationSource = 'manual' | 'trello';
export interface MediaAsset {
  id: string; filename: string; mimeType: string; extension: string; sizeBytes: number;
  storagePath: string; previewPath: string | null; width: number | null; height: number | null;
  duration: number | null; checksum: string; createdAt: string;
}
export interface Publication {
  id: string; title: string; caption: string; format: PublicationFormat; source: PublicationSource;
  editorialStatus: EditorialStatus; version: number; createdAt: string; updatedAt: string; media: MediaAsset[];
}
export interface PublicationSummary extends Omit<Publication, 'caption' | 'version' | 'media'> { mediaCount: number; primaryMedia: MediaAsset | null }
export interface LibraryQuery { search: string; format: PublicationFormat | null; source: PublicationSource | null; status: EditorialStatus | null; sort: 'recent' | 'oldest' | 'title' | 'updated'; offset: number; limit: number }
export interface LibraryPage { items: PublicationSummary[]; total: number }
export interface EditorialCounts { total: number; drafts: number; pendingReview: number; approved: number; ignored: number }
export interface SavePublication { id: string | null; expectedVersion: number | null; title: string; caption: string; format: PublicationFormat; mediaIds: string[] }
export interface ImportResult { media: MediaAsset[]; issues: { filename: string; message: string }[] }
export interface LeaveGuard { dirty: boolean; busy: boolean; discard: () => Promise<void>; save?: () => Promise<boolean> }
export const formatLabels: Record<PublicationFormat, string> = { feed: 'Feed', carousel: 'Carrossel', reels: 'Reels', stories: 'Stories' };
export const statusLabels: Record<EditorialStatus, string> = { draft: 'Rascunho', pending_review: 'Pendente de revisão', editing: 'Em edição', approved: 'Aprovado', ignored: 'Ignorado', ready_to_schedule: 'Pronto para planejar' };
export const sourceLabels: Record<PublicationSource, string> = { manual: 'Manual', trello: 'Trello' };
