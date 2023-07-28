async function handle(req) {
    try {
        console.log("Connection port:", req.port);

        let destIp = "vps.filipton.space:80";
        if (req.port == 7071) {
            return {
                hang_connection: true,
                //block_connection: true, // same as hang_connection but without the 30s sleep
            };
        } else if (req.port == 7072) {
            return {
                ip: destIp,
                no_delay: false, // if you want to proxy more advanced protocols, you need to enable nodelay
            };
        }
    } catch (e) {
        console.error(e.stack);
    }
}
