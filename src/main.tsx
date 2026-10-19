import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './app/App';
import { ErrorBoundary } from './components/ErrorBoundary';
import './styles/tokens.css';
import './styles/global.css';
import './styles/pages.css';
import './styles/editorial.css';
import './styles/calendar.css';

const root = document.getElementById('root');
if (!root) throw new Error('Elemento raiz não encontrado.');
createRoot(root).render(<StrictMode><ErrorBoundary><App /></ErrorBoundary></StrictMode>);
