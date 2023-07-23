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

        return {
            what: "the fuck",
            whats: "going on",
        };
    } catch (e) {
        console.error(e.stack);
    }
}
