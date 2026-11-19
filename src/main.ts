import '@fontsource-variable/hanken-grotesk/index.css';
import { mount } from 'svelte';
import App from './App.svelte';
import './app.css';

import { app } from './lib/state/app.svelte';
import { editor } from './lib/state/editor.svelte';

if (import.meta.env.DEV) Object.assign(window, { renova: app, renovaEditor: editor });

export default mount(App, { target: document.getElementById('app')! });
