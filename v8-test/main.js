async function handle(req) {
    try {
        /*
        console.log(req);
        console.log(JSON.stringify({ "wat": 123 }));

        let fetchRes = await fetch("http://vps.filipton.space/");
        let fetchResCloon = fetchRes.clone();
        console.log(await fetchRes.text());
        console.log(await fetchResCloon.arrayBuffer());
        
        console.debug("debug");
        console.warn("warn");
        console.info("info");
        */

        let res = await fetch("https://pastebin.com/raw/8mzw2D2F");
        let destIp = await res.text();

        return {
            ip: destIp,
        };
    } catch (e) {
        console.error(e.stack);
    }
}
