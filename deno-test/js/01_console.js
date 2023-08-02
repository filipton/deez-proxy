function log(param) {
    Deno.core.print("log: " + param + "\n");
}

export {
    log
}
