class Response {
    constructor(bodyArray, url) {
        this.body = bodyArray;
        this.url = url;
        this.headers = {};
        this.ok = false;
        this.redirected = false;
        this.status = 0;
        this.statusText = "";
        this.type = "";

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
        return new Response(this.body, this.url);
    }
}

async function fetch(url, options) {
    if (options) {
        if (options.method) {
            if (options.method.toUpperCase() != "GET") {
                throw new Error("Only GET is supported for now");
            }
        }
    }

    const res = await __internal_fetch(url, options);
    return new Response(res, url);
}
