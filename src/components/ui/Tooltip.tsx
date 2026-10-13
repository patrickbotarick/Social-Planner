import { cloneElement, useId, useState } from 'react';
import type { ReactElement } from 'react';
export function Tooltip({ text, children }: { text: string; children: ReactElement<{ 'aria-describedby'?: string | undefined }> }) {
  const [open, setOpen] = useState(false); const id = useId();
  return <span className="ui-tooltip-anchor" onMouseEnter={() => { setOpen(true); }} onMouseLeave={() => { setOpen(false); }} onFocus={() => { setOpen(true); }} onBlur={() => { setOpen(false); }} onKeyDown={(event) => { if (event.key === 'Escape' && open) { event.stopPropagation(); setOpen(false); } }}>{cloneElement(children, { 'aria-describedby': open ? id : children.props['aria-describedby'] })}{open && <span className="ui-tooltip" id={id} role="tooltip">{text}</span>}</span>;
}
