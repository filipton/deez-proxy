use color_eyre::Result;
use http_body_util::{combinators::BoxBody, BodyExt, Empty};
use hyper::{
    body::Bytes, server::conn::http1, service::service_fn, upgrade::Upgraded, Method, Request,
    Response,
};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

/*
 * HYPER REVERSE PROXY SERVER
*/

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(&addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(io, service_fn(proxy_service))
                .with_upgrades()
                .await
            {
                println!("server error: {}", e);
            }
        });
    }
}

async fn proxy_service(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>> {
    let addr = "localhost:7070".to_string();
    if req.method() == Method::CONNECT {
        tokio::task::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    if let Err(e) = tunnel(upgraded, addr).await {
                        println!("tunnel error: {}", e);
                    }
                }
                Err(e) => println!("upgrade error: {}", e),
            }
        });

        Ok(Response::new(empty()))
    } else {
        let stream = TcpStream::connect(addr).await?;
        let io = TokioIo::new(stream);

        let (mut sender, conn) = hyper::client::conn::http1::Builder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(io)
            .await?;

        tokio::task::spawn(async move {
            if let Err(e) = conn.await {
                println!("client error: {}", e);
            }
        });

        let resp = sender.send_request(req).await?;
        return Ok(resp.map(|b| b.boxed()));
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

async fn tunnel(upgraded: Upgraded, addr: String) -> std::io::Result<()> {
    let mut server = TcpStream::connect(addr).await?;
    let mut upgraded = TokioIo::new(upgraded);

    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    println!(
        "client wrote {} bytes and received {} bytes",
        from_client, from_server
    );

    Ok(())
}
