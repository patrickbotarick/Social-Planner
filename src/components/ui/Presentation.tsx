import type { HTMLAttributes, ReactNode } from 'react';
import { Icon } from '../Icon';
export function Card({ children, className = '', ...props }: HTMLAttributes<HTMLElement>) { return <section className={`ui-card ${className}`} {...props}>{children}</section>; }
export function Badge({ tone = 'neutral', children }: { tone?: 'neutral' | 'success' | 'warning' | 'danger'; children: ReactNode }) { return <span className={`ui-badge ui-badge--${tone}`}>{children}</span>; }
export function EmptyState({ icon, title, description, action, compact = false }: { icon: string; title: string; description: string; action?: ReactNode; compact?: boolean }) {
  return <div className={`ui-empty ${compact ? 'ui-empty--compact' : ''}`}><Icon name={icon} /><h3>{title}</h3><p>{description}</p>{action && <div className="page-actions">{action}</div>}</div>;
}
export function Toolbar({ label, children }: { label: string; children: ReactNode }) { return <div className="ui-toolbar" role="toolbar" aria-label={label}>{children}</div>; }
export function Skeleton({ width = '100%' }: { width?: string }) { return <div className="ui-skeleton" style={{ width }} aria-hidden="true" />; }
export function MediaThumbnail({ src, kind = 'image', alt, emptyLabel = 'Nenhuma mídia selecionada' }: { src?: string; kind?: 'image' | 'video'; alt: string; emptyLabel?: string }) {
  return <div className="ui-media">{src ? kind === 'video' ? <video src={src} controls preload="metadata" aria-label={alt} /> : <img src={src} alt={alt} /> : <><Icon name="media" /><span>{emptyLabel}</span></>}</div>;
}
