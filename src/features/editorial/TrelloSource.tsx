import { useEffect, useState } from 'react';
import { Card } from '../../components/ui/Presentation';
import { Button } from '../../components/ui/Button';
import { trelloError, trelloSource } from '../../services/trello';
import type { SourceInfo } from '../../types/trello';
import type { MediaAsset } from '../../types/editorial';
import { MediaPreview } from './MediaPreview';
export function TrelloSource({ id, onAdd, disabled }: { id: string; onAdd?: (media: MediaAsset) => void; disabled?: boolean }) {
  const [source, setSource] = useState<SourceInfo | null>(null); const [error, setError] = useState('');
  useEffect(() => { let current = true; void trelloSource(id).then(s => { if (current) setSource(s); }).catch(e => { if (current) setError(trelloError(e)); }); return () => { current = false; }; }, [id]);
  if (error) return <p role="alert">{error}</p>; if (!source) return null;
  return <Card><div className="card-heading"><h2>Origem Trello</h2><a href={source.url} target="_blank" rel="noreferrer">Cartão original ↗</a></div><div className="card-body form-stack">
    {source.sourceChanged && <p className="editorial-notice">A origem mudou desde a importação. O conteúdo editorial local foi preservado.</p>}
    {source.sourceMissing && <p className="editorial-notice">Cartão ausente ou arquivado na origem. Esta publicação permanece disponível.</p>}
    {source.needsAdjustment && <p className="editorial-notice">Confira o formato sugerido, a composição e os anexos. Este material exige ajuste manual antes da revisão.</p>}
    <details><summary>Ver texto original e referências</summary><h3>{source.originalTitle}</h3><div className="caption-content">{source.originalDescription || 'Sem descrição na origem.'}</div><ul className="trello-references">{source.attachments.map((a, i) => <li key={i}><strong>{a.name}</strong> · {a.removed ? 'Removido na origem' : ({ available: 'Disponível localmente', reference: 'Link externo', unsupported: 'Formato não suportado', failed: 'Falha no download' }[a.status] ?? a.status)}{a.errorMessage && <p>{a.errorMessage}</p>}{a.url && <p><a href={a.url} target="_blank" rel="noreferrer">Referência original ↗</a></p>}{a.media && <><MediaPreview media={a.media} thumbnail />{onAdd && <Button variant="secondary" disabled={disabled} onClick={() => { if (a.media) onAdd(a.media); }}>Adicionar à composição: {a.name}</Button>}</>}</li>)}</ul></details>
  </div></Card>;
}
