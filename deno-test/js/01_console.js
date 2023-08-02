function fgw() {
    Deno.core.print("Hello World\n");
}
globalThis.fgw = fgw;
Deno.core.print("Hello World2\n");

export {
    fgw
}
