function paramsToString(params) {
    let res = '';
    for (let i = 0; i < params.length; i++) {
        if (typeof params[i] === 'object') {
            res += JSON.stringify(params[i]) + ' ';
        } else {
            res += `${params[i]} `;
        }
    }
    return res;
}

function log(...params) {
    Deno.core.print(`${paramsToString(params)}\n`);
}

function debug(...params) {
    Deno.core.print(`\x1b[1;94m${paramsToString(params)}\x1b[0;0m\n`);
}

function warn(...params) {
    Deno.core.print(`\x1b[1;93m${paramsToString(params)}\x1b[0;0m\n`);
}

function error(...params) {
    Deno.core.print(`\x1b[1;91m${paramsToString(params)}\x1b[0;0m\n`);
}

export {
    log,
    debug,
    warn,
    error,
    log as info
}
