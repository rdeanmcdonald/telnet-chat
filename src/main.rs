use std::{io, net::SocketAddr};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    join,
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpListener,
    },
    sync::broadcast::{self, Receiver, Sender},
};

type Message = (String, SocketAddr);

async fn receive_channel_message(
    mut receiver: Receiver<Message>,
    soc_sender: tokio::sync::mpsc::Sender<Message>,
) {
    loop {
        let msg = receiver.recv().await.unwrap();
        soc_sender.send(msg).await.unwrap();
    }
}

async fn receive_client_message<'a>(
    reader: ReadHalf<'a>,
    my_addr: SocketAddr,
    sender: Sender<Message>,
) {
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();
    loop {
        let bytes_read = buf_reader.read_line(&mut line).await.unwrap();

        if bytes_read == 0 {
            break;
        }

        sender.send((line.clone(), my_addr)).unwrap();
        line.clear();
    }
}

async fn socket_writer<'a>(
    mut writer: WriteHalf<'a>,
    mut soc_receiver: tokio::sync::mpsc::Receiver<Message>,
    my_addr: SocketAddr,
) {
    loop {
        let msg = soc_receiver.recv().await;
        match msg {
            Some((m, addr)) => {
                if addr != my_addr {
                    writer.write_all(m.as_bytes()).await.unwrap();
                }
            }
            None => break,
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let (brod_sender, _recevier) = broadcast::channel::<Message>(10);

    loop {
        // every iteration represenets a new connection
        let brod_sender = brod_sender.clone();
        let brod_receiver = brod_sender.subscribe();
        let (mut socket, my_addr) = listener.accept().await?;
        tokio::spawn(async move {
            // A CONNECTION

            let (soc_sender, soc_receiver) = tokio::sync::mpsc::channel::<Message>(10);

            // the reader/writer are just refs to the underlying TcpStream how
            // can we share this TcpStream ref across threads? E.g. how can we
            // have one thread read, and another write? Not exactly sure, but
            // somehow ReadHalf/WriteHalf impl Send/Sync, so it works
            let (soc_reader, soc_writer) = socket.split();

            // In parallel, receive message from client, and receive message
            // from channel
            let (_a, _b, _c) = join!(
                receive_client_message(soc_reader, my_addr, brod_sender.clone()),
                receive_channel_message(brod_receiver, soc_sender),
                socket_writer(soc_writer, soc_receiver, my_addr)
            );
        });
    }
}
