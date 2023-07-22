
async function run(a, b) {
    try {
        console.log(JSON.stringify({ "wat": 123 }));

        let fetchRes = await fetch("http://vps.filipton.space/");
        let fetchResCloon = fetchRes.clone();
        console.log(await fetchRes.text());
        console.log(await fetchResCloon.arrayBuffer());

        return {
            what: "the fuck",
            whats: "going on",
            age: a + b
        };
    } catch (e) {
        console.error(e);
    }
}
