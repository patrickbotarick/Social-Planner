import type { MediaAsset } from './editorial';
export interface ConnectionStatus { configured: boolean; account: { fullName: string; username: string } | null }
export interface Attachment { id: string; name: string; url: string; mimeType: string | null; bytes: number; isUpload: boolean }
export interface RemoteCard { id: string; name: string; desc: string; idList: string; shortLink: string; closed: boolean; attachments: Attachment[] }
export interface PreviewCard { card: RemoteCard; state: 'new' | 'changed' | 'imported' | 'archived'; publicationId: string | null }
export interface BoardPreview { previewId: string; board: { id: string; name: string; shortLink: string }; lists: { id: string; name: string; closed: boolean }[]; cards: PreviewCard[]; lastSyncedAt: string | null }
export interface DownloadLimits { imageMib: number; videoMib: number; timeoutSeconds: number }
export interface ImportJob { id: string; status: string; totalCards: number; processedCards: number; successfulCards: number; failedCards: number; newPublications: number; changedSources: number; downloadedAttachments: number; attachmentFailures: number; errorMessage: string | null; currentCard: string | null; downloadedBytes: number }
export interface ImportHistory { boards: { name: string; url: string; lastSyncedAt: string | null }[]; jobs: ImportJob[]; active: ImportJob | null }
export interface SourceInfo { url: string; originalTitle: string; originalDescription: string; sourceChanged: boolean; sourceMissing: boolean; needsAdjustment: boolean; attachments: { name: string; url: string; status: string; errorMessage: string | null; removed: boolean; media: MediaAsset | null }[] }
