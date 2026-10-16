import { useEffect, useId, useRef } from 'react';
import type { ReactNode } from 'react';
import { Button, IconButton } from './Button';
import { Icon } from '../Icon';
export function Modal({ open, title, onClose, children, actions }: { open: boolean; title: string; onClose: () => void; children: ReactNode; actions?: ReactNode }) {
  const dialog = useRef<HTMLDialogElement>(null); const heading = useId();
  useEffect(() => {
    const element = dialog.current;
    if (!element || !open) return;
    const opener = document.activeElement;
    element.showModal();
    element.querySelector('button')?.focus();
    return () => { element.close(); if (opener instanceof HTMLElement && opener.isConnected) opener.focus(); };
  }, [open]);
  return <dialog ref={dialog} className="ui-dialog" tabIndex={-1} aria-labelledby={heading} onCancel={(event) => { event.preventDefault(); onClose(); }} onKeyDown={(event) => {
    if (event.key !== 'Tab') return;
    const controls = event.currentTarget.querySelectorAll<HTMLElement>('button:not([disabled]), a[href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex="0"]');
    const first = controls[0]; const last = controls[controls.length - 1];
    if (!first) { event.preventDefault(); event.currentTarget.focus(); }
    else if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
    else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
  }}><div className="dialog-heading"><h2 id={heading}>{title}</h2><IconButton label="Fechar diálogo" onClick={onClose}><Icon name="close" /></IconButton></div><div className="dialog-content">{children}</div>{actions && <div className="dialog-actions">{actions}</div>}</dialog>;
}
// Confirmação reutilizada por ações editoriais explícitas.
export function ConfirmDialog({ open, title, description, confirmLabel, onConfirm, onClose }: { open: boolean; title: string; description: string; confirmLabel: string; onConfirm: () => void; onClose: () => void }) {
  return <Modal open={open} title={title} onClose={onClose} actions={<><Button variant="secondary" onClick={onClose}>Cancelar</Button><Button variant="danger" onClick={onConfirm}>{confirmLabel}</Button></>}><p>{description}</p></Modal>;
}
export function UnsavedDialog({open,busy,onStay,onDiscard,onSave}:{open:boolean;busy:boolean;onStay:()=>void;onDiscard:()=>void;onSave?:(()=>void)|undefined}) {
  return <Modal open={open} title="Alterações não salvas" onClose={()=>{if(!busy)onStay();}} actions={<><Button variant="secondary" disabled={busy} onClick={onStay}>Permanecer</Button><Button variant="danger" disabled={busy} onClick={onDiscard}>Descartar e sair</Button>{onSave && <Button disabled={busy} onClick={onSave}>Salvar e continuar</Button>}</>}><p>Escolha como continuar. Os arquivos originais e as publicações já salvas serão preservados.</p></Modal>;
}
