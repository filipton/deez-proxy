async function run(req) {
    if (req.port == 25565) {
        return {
            ip: "vps.filipton.space:25565",
            no_delay: true
        }
    }

    return {
        ip: "localhost:80",
    }
}
