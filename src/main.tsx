import { render } from '@solidjs/web';
import App from './App';
import { installHardening } from './lib/hardening';
import './index.css';

// Dev builds keep the default behaviour for inspection.
if (import.meta.env.PROD) installHardening();

const root = document.getElementById('root');

if (!root) {
  throw new Error('Mount target #root is missing — index.html must contain <div id="root"></div>.');
}

render(() => <App />, root);
