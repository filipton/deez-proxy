use color_eyre::{eyre::eyre, Result};
use tokio::net::{TcpListener, TcpStream};
use v8_utils::V8Response;

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
                    let code = tokio::fs::read_to_string("./main.js").await.unwrap();
                    let res = v8_utils::get_script_res(&code, addr).await.unwrap();
                    if res.block.unwrap_or(false) {
                        println!("Blocking: {}", addr);
                        return;
                    } else if res.hang_connection.unwrap_or(false) {
                        println!("Hanging: {}", addr);
                        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                        return;
                    }

                    if let Err(e) = handle_client(socket, res).await {
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

async fn handle_client(mut socket: TcpStream, output: V8Response) -> Result<()> {
    let mut out_stream =
        TcpStream::connect(output.ip.ok_or(eyre!("Ip is null in V8Response"))?).await?;
    out_stream.set_nodelay(output.no_delay.unwrap_or(false))?;

    tokio::io::copy_bidirectional(&mut socket, &mut out_stream).await?;

    Ok(())
}
