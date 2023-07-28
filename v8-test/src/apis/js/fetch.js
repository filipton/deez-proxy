class Response {
    constructor(resp) {
        this.body = new ReadableStream();
        if (resp?.body) {
            for (let byte of resp.body) {
                this.body.enqueue(byte);
            }
        }

        this.headers = resp?.headers || new Headers();
        this.ok = resp?.ok || false;
        this.redirected = resp?.redirected || false;
        this.status = resp?.status || 0;
        this.statusText = resp?.statusText || "";
        this.type = resp?.type || "basic"; // NOT USED
        this.url = resp?.url || "";

        Object.defineProperty(this, "bodyUsed", {
            value: false,
            writable: false,
            configurable: true
        });
    }

    async text() {
        if (this.bodyUsed) {
            throw new Error("Body already used");
        }

        Object.defineProperty(this, "bodyUsed", {
            value: true,
        });

        let body = new Uint8Array();
        let reader = this.body.getReader();
        let result = await reader.read();
        while (!result.done) {
            body = new Uint8Array([...body, result.value]);
            result = await reader.read();
        }

        return new TextDecoder().decode(body);
    }

    async json() {
        if (this.bodyUsed) {
            throw new Error("Body already used");
        }

        Object.defineProperty(this, "bodyUsed", {
            value: true,
        });

        let body = new Uint8Array();
        let reader = this.body.getReader();
        let result = await reader.read();
        while (!result.done) {
            body = new Uint8Array([...body, result.value]);
            result = await reader.read();
        }

        return JSON.parse(new TextDecoder().decode(body));
    }

    async arrayBuffer() {
        if (this.bodyUsed) {
            throw new Error("Body already used");
        }

        Object.defineProperty(this, "bodyUsed", {
            value: true,
        });

        let body = new Uint8Array();
        let reader = this.body.getReader();
        let result = await reader.read();
        while (!result.done) {
            body = new Uint8Array([...body, result.value]);
            result = await reader.read();
        }

        return new ArrayBuffer(body);
    }

    clone() {
        let res = new Response();
        res.body = new ReadableStream();
        for (let byte of this.body.queue) {
            res.body.enqueue(byte);
        }

        res.headers = this.headers;
        res.ok = this.ok;
        res.redirected = this.redirected;
        res.status = this.status;
        res.statusText = this.statusText;
        res.type = this.type;
        res.url = this.url;

        return res;
    }
}

class Headers {
    constructor() {
        this.headers = {};
    }

    append(key, value) {
        this.headers[key] = value;
    }

    delete(key) {
        delete this.headers[key];
    }

    get(key) {
        return this.headers[key];
    }

    has(key) {
        return this.headers[key] != undefined;
    }

    set(key, value) {
        this.headers[key] = value;
    }

    entries() {
        return Object.entries(this.headers);
    }

    keys() {
        return Object.keys(this.headers);
    }

    values() {
        return Object.values(this.headers);
    }

    [Symbol.iterator]() {
        return Object.entries(this.headers)[Symbol.iterator]();
    }
}

class Request {
    constructor(url, options) {
        this.body = new ReadableStream();
        if (options?.body) {
            if (typeof options.body == "string") {
                for (let byte of new TextEncoder().encode(options.body)) {
                    this.body.enqueue(byte);
                }
            } else if (options.body instanceof ArrayBuffer) {
                for (let byte of options.body) {
                    this.body.enqueue(byte);
                }
            } else {
                throw new Error("Invalid body type");
            }
        }

        this.cache = options?.cache || "default"; // NOT USED
        this.credentials = options?.credentials || "omit"; // NOT USED
        this.destination = options?.destination || "document"; // NOT USED
        this.headers = options?.headers || new Headers();
        this.integrity = options?.integrity || ""; // NOT USED
        this.method = options?.method || "GET";
        this.mode = options?.mode || "no-cors"; // NOT USED
        this.redirect = options?.redirect || "follow"; // NOT USED
        this.referrer = options?.referrer || ""; // NOT USED
        this.referrerPolicy = options?.referrerPolicy || ""; // NOT USED
        this.signal = options?.signal || ""; // NOT USED
        this.url = url;

        Object.defineProperty(this, "bodyUsed", {
            value: false,
            writable: false,
            configurable: true
        });
    }

    async text() {
        if (this.bodyUsed) {
            throw new Error("Body already used");
        }

        Object.defineProperty(this, "bodyUsed", {
            value: true,
        });
        return new TextDecoder().decode(this.body);
    }

    async json() {
        if (this.bodyUsed) {
            throw new Error("Body already used");
        }

        Object.defineProperty(this, "bodyUsed", {
            value: true,
        });
        return JSON.parse(new TextDecoder().decode(this.body));
    }

    async arrayBuffer() {
        if (this.bodyUsed) {
            throw new Error("Body already used");
        }

        Object.defineProperty(this, "bodyUsed", {
            value: true,
        });
        return new ArrayBuffer(this.body);
    }

    clone() {
        let req = new Request(this.url, {
            body: this.body,
            cache: this.cache,
            credentials: this.credentials,
            destination: this.destination,
            headers: this.headers,
            integrity: this.integrity,
            method: this.method,
            mode: this.mode,
            redirect: this.redirect,
            referrer: this.referrer,
            referrerPolicy: this.referrerPolicy,
            signal: this.signal,
        });

        return req;
    }
}

class InnerFetchRequest {
    constructor(headers, method, url) {
        this.headers = headers;
        this.method = method;
        this.url = url;
    }
}

async function fetch(url, options) {
    let req = new Request(url, options);

    let body = new Uint8Array();
    let reader = req.body.getReader();
    let result = await reader.read();
    while (!result.done) {
        body = new Uint8Array([...body, result.value]);
        result = await reader.read();
    }

    let innerReq = new InnerFetchRequest(req.headers.headers, req.method, req.url);
    return new Response(await __internal_fetch(innerReq, body.buffer));
}
