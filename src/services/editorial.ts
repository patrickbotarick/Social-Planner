import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import type { EditorialCounts, ImportResult, LibraryPage, LibraryQuery, MediaAsset, Publication, PublicationFormat, SavePublication } from '../types/editorial';
import { describeDesktopError } from './desktop';
export const listPublications = (query: LibraryQuery) => invoke<LibraryPage>('publication_list', { query });
export const getPublication = (id: string) => invoke<Publication>('publication_get', { id });
export const savePublication = (input: SavePublication) => invoke<Publication>('publication_save', { input });
export const duplicatePublication = (id: string) => invoke<Publication>('publication_duplicate', { id });
export const deletePublication = (id: string, expectedVersion: number) => invoke<void>('publication_delete', { id, expectedVersion });
export const getEditorialCounts = () => invoke<EditorialCounts>('editorial_counts');
export const selectMedia = (format: PublicationFormat) => invoke<ImportResult>('media_select', { format });
export const discardMedia = (ids: string[]) => ids.length ? invoke<void>('media_discard', { ids }) : Promise.resolve();
export const mediaUrl = (media: MediaAsset) => media.previewPath ? convertFileSrc(media.previewPath) : undefined;
export function editorialError(error: unknown): string {
  if (typeof error === 'object' && error !== null && 'code' in error && 'message' in error && ['VALIDATION','MEDIA_INVALID','MEDIA_IO','NOT_FOUND','EDIT_CONFLICT','REVIEW_INVALID','CALENDAR_INVALID','ASSISTED_INVALID','EXPORT_PATH','EXPORT_SPACE','EXPORT_IO','EXPORT_COLLISION','EXPORT_OUTDATED','EXPORT_OPEN'].includes(String(error.code)) && typeof error.message === 'string') return error.message;
  return describeDesktopError(error);
}
