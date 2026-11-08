import { mount } from 'svelte';
import App from './App.svelte';
import './app.css';

const target = document.getElementById('app');
let app = null;
if (target) {
  target.innerHTML = '';
  app = mount(App, { target });
}
export default app;
