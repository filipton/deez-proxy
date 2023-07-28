async function handle(req) {
    try {
        console.log("Connection port:", req.port);

        if (req.port == 7071) {
            return {
                hang_connection: true,
                //block_connection: true, // same as hang_connection but without the 30s sleep
            };
        } else if (req.port == 7072) {
            console.log(fetch.dsa());
            return {
                ip: "192.168.1.1:80",
                no_delay: false, // if you want to proxy more advanced protocols, you need to enable nodelay
            };
        }
    } catch (e) {
        console.error(e.stack);
    }
}
