class Response {
    constructor() {
        this.body = "";
        this.headers = new Headers();
        this.ok = false;
        this.redirected = false;
        this.status = 0;
        this.statusText = "";
        this.type = "";
        this.url = "";

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
        let res = new Response();
        res.body = this.body;
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
        this.body = options.body || null;
        this.cache = options.cache || "default"; // NOT USED
        this.credentials = options.credentials || "omit"; // NOT USED
        this.destination = options.destination || "document"; // NOT USED
        this.headers = options.headers || new Headers();
        this.integrity = options.integrity || ""; // NOT USED
        this.method = options.method || "GET";
        this.mode = options.mode || "no-cors"; // NOT USED
        this.redirect = options.redirect || "follow"; // NOT USED
        this.referrer = options.referrer || ""; // NOT USED
        this.referrerPolicy = options.referrerPolicy || ""; // NOT USED
        this.signal = options.signal || ""; // NOT USED
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

async function fetch(url, options) {
    let req = new Request(url, options);
    return await __internal_fetch(req);
}
