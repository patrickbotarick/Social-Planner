import { useId, useRef, useState } from 'react';
import type { ReactNode } from 'react';
export function Tabs({ label, items }: { label: string; items: ReadonlyArray<{ id: string; label: string; content: ReactNode }> }) {
  const [selected, setSelected] = useState(items[0]?.id); const prefix = useId(); const buttons = useRef<(HTMLButtonElement | null)[]>([]);
  return <div><div role="tablist" aria-label={label} className="ui-tabs">{items.map((item, index) => <button key={item.id} ref={(element) => { buttons.current[index] = element; }} id={`${prefix}-${item.id}`} role="tab" className="ui-tab" aria-selected={selected === item.id} aria-controls={`${prefix}-${item.id}-panel`} tabIndex={selected === item.id ? 0 : -1} onClick={() => { setSelected(item.id); }} onKeyDown={(event) => {
    if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
    event.preventDefault();
    const next = event.key === 'Home' ? 0 : event.key === 'End' ? items.length - 1 : (index + (event.key === 'ArrowRight' ? 1 : -1) + items.length) % items.length;
    const nextItem = items[next]; if (nextItem) { setSelected(nextItem.id); buttons.current[next]?.focus(); }
  }}>{item.label}</button>)}</div>{items.map((item) => <div key={item.id} role="tabpanel" id={`${prefix}-${item.id}-panel`} aria-labelledby={`${prefix}-${item.id}`} hidden={selected !== item.id} tabIndex={0} className="tab-panel">{item.content}</div>)}</div>;
}
