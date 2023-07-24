async function handle(req) {
    try {
        console.error(JSON.stringify(req));
        
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

        /*
        let res = await fetch("https://files.usbus.space/test.txt");
        let destIp = await res.text();
        */
        let destIp = "vps.filipton.space:80";

        return {
            ip: destIp,
        };
    } catch (e) {
        console.error(e.stack);
    }
}
