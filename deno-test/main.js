async function run(req) {
    if (req.port == 25565) {
        return {
            ip: "localhost:25566",
            no_delay: true
        }
    }

    return {
        ip: "localhost:80",
    }
}
