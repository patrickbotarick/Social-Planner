import { Component } from 'react';
import type { ReactNode } from 'react';

export class ErrorBoundary extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false };
  static getDerivedStateFromError() { return { failed: true }; }
  render() {
    if (this.state.failed) return <main className="fatal" role="alert"><h1>Não foi possível exibir o aplicativo</h1><p>Reabra a interface para tentar novamente. Os dados locais não serão apagados.</p><button onClick={() => window.location.reload()}>Reabrir interface</button></main>;
    return this.props.children;
  }
}
