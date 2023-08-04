import * as console from 'ext:console/console.js';

delete globalThis.console;
globalThis.console = console;
