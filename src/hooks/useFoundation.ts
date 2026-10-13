import { useCallback, useEffect, useRef, useState } from 'react';
import { describeDesktopError, desktopAvailable, getFoundationStatus } from '../services/desktop';
import type { FoundationState } from '../types/foundation';

export function useFoundation() {
  const [state, setState] = useState<FoundationState>({ kind: 'loading' });
  const request = useRef(0);
  const refresh = useCallback(async () => {
    const current = ++request.current;
    if (!desktopAvailable()) { setState({ kind: 'browser' }); return; }
    setState({ kind: 'loading' });
    try {
      const data = await getFoundationStatus();
      if (current === request.current) setState({ kind: 'ready', data });
    } catch (error: unknown) {
      if (current === request.current) setState({ kind: 'error', message: describeDesktopError(error) });
    }
  }, []);
  useEffect(() => {
    void refresh();
    return () => { request.current += 1; };
  }, [refresh]);
  return { state, refresh };
}
