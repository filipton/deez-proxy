
async function run(a, b) {
    try {
        console.log(JSON.stringify({ "wat": 123 }));

        let fetchRes = await fetch("http://vps.filipton.space/");
        console.log("fetchRes", fetchRes);
        console.log(await fetchRes.text());

        return {
            what: "the fuck",
            whats: "going on",
            age: a + b
        };
    } catch (e) {
        console.error(e);
    }
}
