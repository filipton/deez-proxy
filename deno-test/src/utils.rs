use color_eyre::Result;

pub fn parse_ports(ports: &str) -> Result<Vec<u16>> {
    let mut res = vec![];
    for port in ports.split(',') {
        let port = port.trim();
        if port.is_empty() {
            continue;
        }

        if port.contains('-') {
            let mut range = port.split('-');
            let start = range.next().unwrap().trim().parse::<u16>()?;
            let end = range.next().unwrap().trim().parse::<u16>()?;
            for port in start..=end {
                res.push(port);
            }
        } else {
            let port = port.parse::<u16>()?;
            res.push(port);
        }
    }
    Ok(res)
}
