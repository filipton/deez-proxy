async function handle(req) {
    try {
        console.log("Connection port: ", req.port);

        let destIp = "vps.filipton.space:80";
        if (req.port == 7071) {
            return {
                hang_connection: true,
            };
        } else if (req.port == 7072) {
            return {
                ip: destIp,
                no_delay: true,
            };
        }
    } catch (e) {
        console.error(e.stack);
    }
}
