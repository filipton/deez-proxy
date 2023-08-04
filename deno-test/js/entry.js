import * as console from 'ext:console/console.js';
import * as others from 'ext:others/others.js';
import * as fetch from 'ext:fetch/fetch.js';

globalThis.console = console;

globalThis.TextEncoder = others.TextEncoder;
globalThis.TextDecoder = others.TextDecoder;

globalThis.fetch = fetch.fetch;
globalThis.Response = fetch.Response;
globalThis.Request = fetch.Request;
globalThis.Headers = fetch.Headers;
