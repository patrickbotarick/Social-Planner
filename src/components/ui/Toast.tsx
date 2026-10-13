import { createContext, useCallback, useContext, useEffect, useRef, useState } from 'react';
import type { ReactNode } from 'react';
import { IconButton } from './Button';
import { Icon } from '../Icon';
type ToastItem = { id: number; message: string; tone: 'success' | 'danger' };
const ToastContext = createContext<(message: string, tone?: ToastItem['tone']) => void>(() => {});
export function ToastProvider({ children }: { children: ReactNode }) {
  const [toast, setToast] = useState<ToastItem | null>(null); const sequence = useRef(0);
  const notify = useCallback((message: string, tone: ToastItem['tone'] = 'success') => { setToast({ id: ++sequence.current, message, tone }); }, []);
  useEffect(() => { if (!toast || toast.tone === 'danger') return; const timeout = window.setTimeout(() => { setToast(null); }, 6000); return () => window.clearTimeout(timeout); }, [toast]);
  return <ToastContext.Provider value={notify}>{children}{toast && <div key={toast.id} role={toast.tone === 'danger' ? 'alert' : 'status'} className={`ui-toast ui-toast--${toast.tone}`}><span>{toast.message}</span><IconButton label="Fechar aviso" onClick={() => { setToast(null); }}><Icon name="close" /></IconButton></div>}</ToastContext.Provider>;
}
export const useToast = () => useContext(ToastContext);
