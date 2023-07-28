use color_eyre::{eyre::eyre, Result};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

mod apis;
mod utils;
mod v8_utils;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    v8_utils::install();

    let mut tasks = vec![];

    tasks.push(tokio::spawn(port_worker("0.0.0.0", 7071)));
    tasks.push(tokio::spawn(port_worker("0.0.0.0", 7072)));

    futures::future::try_join_all(tasks).await?;

    Ok(())
}

async fn port_worker(bind_ip: &str, port: u16) -> Result<()> {
    let addr = format!("{}:{}", bind_ip, port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        let socket_res = listener.accept().await;

        match socket_res {
            Ok((socket, addr)) => {
                tokio::spawn(async move {
                    if let Err(e) = handle_client(socket, port, addr).await {
                        println!("Handle Client Error: {}", e);
                    }
                });
            }
            Err(e) => {
                println!("Bind Socket Error: {}", e);
            }
        }
    }
}

async fn handle_client(mut socket: TcpStream, port: u16, addr: SocketAddr) -> Result<()> {
    let code = tokio::fs::read_to_string("./main.js").await?;

    let res = v8_utils::get_script_res(&code, port, addr).await?;
    if res.block_connection.unwrap_or(false) {
        return Ok(());
    } else if res.hang_connection.unwrap_or(false) {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        return Ok(());
    }

    let mut out_stream =
        TcpStream::connect(res.ip.ok_or(eyre!("Ip is null in V8Response"))?).await?;
    out_stream.set_nodelay(res.no_delay.unwrap_or(false))?;

    tokio::io::copy_bidirectional(&mut socket, &mut out_stream).await?;

    Ok(())
}
