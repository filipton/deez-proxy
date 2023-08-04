use crate::workers::JOB_QUEUE;
use color_eyre::Result;
use structs::V8Response;
use tokio::net::TcpStream;
use workers::port_listener;
use workers::v8_worker;

mod extensions;
mod structs;
mod utils;
mod workers;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let workers_count = 10usize;
    let mut workers = vec![];
    for i in 0..workers_count {
        workers.push(std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            loop {
                let res = v8_worker(&rt, i);
                if let Err(e) = res {
                    println!("Worker {} failed: {:?}", i, e);
                }
            }
        }));
    }

    let args = std::env::args().collect::<Vec<String>>();
    let ports = utils::parse_ports(args.get(1).unwrap_or(&String::from("7070")))?;
    let mut tasks = vec![];

    for port in ports {
        tasks.push(tokio::spawn(port_listener("0.0.0.0", port)));
    }
    futures::future::try_join_all(tasks).await?;

    Ok(())
}

async fn handle_client(mut socket: TcpStream, res: V8Response) -> Result<()> {
    if res.block_connection.unwrap_or(false) {
        return Ok(());
    } else if res.hang_connection.unwrap_or(false) {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        return Ok(());
    }

    let mut out_stream = TcpStream::connect(
        res.ip
            .ok_or(color_eyre::eyre::eyre!("Ip is null in V8Response"))?,
    )
    .await?;
    out_stream.set_nodelay(res.no_delay.unwrap_or(false))?;

    tokio::io::copy_bidirectional(&mut socket, &mut out_stream).await?;
    Ok(())
}
