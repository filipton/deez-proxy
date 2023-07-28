async function handle(req) {
    try {
        let fetchRes = await fetch("https://echo.filipton.space/r16073722548685778558", { body: "test" });
        let cloned = fetchRes.clone();
        console.log(await fetchRes.text());
        console.log("cloned: ", await cloned.text());

        /*
        let res = await fetch("https://files.usbus.space/test.txt");
        let destIp = await res.text();
        */
        let destIp = "vps.filipton.space:80";

        return {
            block: true,
            ip: destIp,
            no_delay: true,
        };
    } catch (e) {
        console.error(e.stack);
    }
}
