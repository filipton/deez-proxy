import * as console from 'ext:console/01_console.js';

delete globalThis.console;
globalThis.console = console;
