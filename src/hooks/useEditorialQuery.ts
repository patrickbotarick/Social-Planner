import { useCallback, useEffect, useRef, useState } from 'react';
import { desktopAvailable } from '../services/desktop';
import { editorialError } from '../services/editorial';
export type QueryState<T> = { kind: 'loading' } | { kind: 'browser' } | { kind: 'error'; message: string } | { kind: 'ready'; data: T };
export function useEditorialQuery<T>(request: () => Promise<T>) {
  const [state, setState] = useState<QueryState<T>>({ kind: 'loading' });
  const sequence = useRef(0);
  const refresh = useCallback(async () => {
    const current = ++sequence.current;
    if (!desktopAvailable()) { setState({ kind: 'browser' }); return; }
    setState({ kind: 'loading' });
    try { const data = await request(); if (current === sequence.current) setState({ kind: 'ready', data }); }
    catch (error) { if (current === sequence.current) setState({ kind: 'error', message: editorialError(error) }); }
  }, [request]);
  useEffect(() => { void refresh(); return () => { sequence.current += 1; }; }, [refresh]);
  return { state, refresh };
}
