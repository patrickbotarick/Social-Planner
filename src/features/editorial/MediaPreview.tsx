import { useState } from 'react';
import { Icon } from '../../components/Icon';
import { mediaUrl } from '../../services/editorial';
import type { MediaAsset } from '../../types/editorial';
export function MediaPreview({ media, thumbnail = false, onUnavailable }: { media: MediaAsset; thumbnail?: boolean; onUnavailable?: () => void }) {
  const [failed, setFailed] = useState(false);
  const url = mediaUrl(media);
  return <div className={'editorial-media-preview' + (thumbnail ? ' editorial-media-preview--thumbnail' : '')}>
    {failed || !url ? <div className="media-unavailable"><Icon name="media" /><span>Mídia indisponível</span></div>
      : media.mimeType.startsWith('video/') ? thumbnail ? <div className="video-thumbnail"><Icon name="media" /><span>Vídeo · MP4</span></div> : <video src={url} controls preload="metadata" aria-label={media.filename} onError={() => { setFailed(true); onUnavailable?.(); }} />
        : <img src={url} alt={media.filename} loading={thumbnail ? 'lazy' : 'eager'} decoding="async" onError={() => { setFailed(true); onUnavailable?.(); }} />}
  </div>;
}
export function fileSize(bytes: number) { return bytes < 1024 * 1024 ? (bytes / 1024).toLocaleString('pt-BR', { maximumFractionDigits: 1 }) + ' KB' : (bytes / (1024 * 1024)).toLocaleString('pt-BR', { maximumFractionDigits: 1 }) + ' MB'; }
export function editorialDate(value: string) { return new Intl.DateTimeFormat('pt-BR', { dateStyle: 'short', timeStyle: 'short' }).format(new Date(value)); }
