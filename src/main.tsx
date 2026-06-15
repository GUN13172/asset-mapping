import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { startPerf } from './utils/perf';
import './styles/theme.css';
import './styles/theme-light.css';
import './styles/animations.css';
import './styles/components.css';
import './styles/app.css';
import './styles.css';

const appBootstrapPerfToken = startPerf('app-bootstrap', {
  stage: 'main-entry',
});

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App bootstrapPerfToken={appBootstrapPerfToken} />
  </React.StrictMode>
);
