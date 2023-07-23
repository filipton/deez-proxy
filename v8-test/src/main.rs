use color_eyre::Result;
use tokio::net::{TcpListener, TcpStream};

mod apis;
mod utils;
mod v8_utils;

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct TestStruct {
    pub what: String,
    pub whats: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    v8_utils::install();

    let mut tasks = vec![];

    tasks.push(tokio::spawn(port_worker(7071)));
    tasks.push(tokio::spawn(port_worker(7072)));

    futures::future::try_join_all(tasks).await?;

    Ok(())
}

async fn port_worker(port: u16) -> Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        let socket_res = listener.accept().await;

        match socket_res {
            Ok((socket, _addr)) => {
                tokio::spawn(async move {
                    let code = tokio::fs::read_to_string("./main.js").await.unwrap();

                    if let Err(e) =
                        handle_client(socket, v8_utils::get_script_res(&code).await.unwrap()).await
                    {
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

async fn handle_client(mut socket: TcpStream, output: TestStruct) -> Result<()> {
    println!("Output: {:?}", output);

    let mut out_stream = TcpStream::connect("192.168.1.1:80").await?;
    tokio::io::copy_bidirectional(&mut socket, &mut out_stream).await?;

    Ok(())
}
