async function handle(req) {
    try {
        //await sleep(1);
        if (req.port == 7071) {
            return {
                hang_connection: true,
                //block_connection: true, // same as hang_connection but without the 30s sleep
            }
        } else if (req.port == 7070) {
            return {
                ip: "localhost:80",
                no_delay: true, // if you want to proxy more advanced protocols, you need to enable nodelay
            }
        }

        return {
            ip: "vps.filipton.space:25565",
            no_delay: true,
        }
    } catch (e) {
        console.error(e.stack);
    }
}
