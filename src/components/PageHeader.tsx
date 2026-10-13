import type { ReactNode } from 'react';
export function PageHeader({ title, description, actions }: { title: string; description: string; actions?: ReactNode }) { return <div className="page-header"><div><h1>{title}</h1><p>{description}</p></div>{actions && <div className="page-actions">{actions}</div>}</div>; }
export function AvailabilityNote({ phase, children }: { phase: string; children: ReactNode }) { return <p className="phase-note"><strong>Disponível na Fase {phase}</strong><span>{children}</span></p>; }
