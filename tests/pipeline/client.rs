use crate::{unwrap, ready, PanicError, Request, Response};
use async_bincode::*;
use tokio;
use tokio_tower::pipeline::Client;
use tower_service::Service;
use futures::sink::SinkExt;
use futures::stream::StreamExt;

#[tokio::test]
async fn it_works() {
    let mut rx = tokio::net::tcp::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = rx.local_addr().unwrap();

    // connect
    let tx = tokio::net::tcp::TcpStream::connect(&addr).await.unwrap();
    let tx: AsyncBincodeStream<_, Response, _, _> = AsyncBincodeStream::from(tx).for_async();
    let mut tx: Client<_, PanicError, _> = Client::new(tx);

    tokio::spawn(async move {
        loop {
            let (stream, _) = rx.accept().await.unwrap();
            let (r, w) = stream.split();
            let mut r: AsyncBincodeReader<_, Request> = AsyncBincodeReader::from(r);
            let mut w: AsyncBincodeWriter<_, Response, _> = AsyncBincodeWriter::from(w).for_async();
            tokio::spawn(
                async move {
                    loop {
                        let req = r.next().await.unwrap().unwrap();
                        w.send(Response::from(req)).await.unwrap();
                    }
                }
            );
        }
    });

    unwrap(ready(&mut tx).await);
    unwrap(tx.call(Request::new(1)).await).check(1);
}
