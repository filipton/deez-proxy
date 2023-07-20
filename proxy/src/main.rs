use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:7070";

    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        let socket_res = listener.accept().await;

        match socket_res {
            Ok((socket, _addr)) => {
                tokio::spawn(async move {
                    if let Err(e) = handle_client(socket).await {
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

async fn handle_client(mut socket: TcpStream) -> Result<()> {
    let mut out_stream = TcpStream::connect("192.168.1.1:80").await?;

    let mut in_buf = vec![0; 4096];
    let mut out_buf = vec![0; 4096];

    loop {
        tokio::select! {
            res = socket.read(&mut in_buf) => {
                if let Ok(n) = res {
                    if n == 0 {
                        break;
                    }
                    let res = out_stream.write_all(&in_buf[..n]).await;

                    if let Err(e) = res {
                        println!("In Error: {}", e);
                        break;
                    }
                }
            }
            res = out_stream.read(&mut out_buf) => {
                if let Ok(n) = res {
                    if n == 0 {
                        break;
                    }
                    let res = socket.write_all(&out_buf[..n]).await;

                    if let Err(e) = res {
                        println!("Out Error: {}", e);
                        break;
                    }
                }
            }
            else => {
                println!("Else");
            }
        }
    }

    Ok(())
}
