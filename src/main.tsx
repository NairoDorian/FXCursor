import { render } from '@solidjs/web';
import App from './App';
import './index.css';

const root = document.getElementById('root');

if (!root) {
  throw new Error('Mount target #root is missing — index.html must contain <div id="root"></div>.');
}

render(() => <App />, root);
