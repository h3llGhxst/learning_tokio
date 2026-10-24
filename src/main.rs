
use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};


#[tokio::main] 
async fn main() { 
    // bind the listener to teh adress 
    
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        //second item contains the IP and port onf the new conn
        let (socket, _) = listener.accept().await.unwrap();
    
        //spawn a new task for each inbound socket , the new socket is moved 
        // to the new task and processed there.

        tokio::spawn(async move{ 
            process(socket).await;
        });  
    }
}

async fn process(socket: TcpStream) { 

    let mut connection = Connection::new(socket); 

    if let Some(frame) = connection.read_frame().await.unwrap() { 
        println!("GOT : {:?}", frame);

        let response = Frame::Error("unimplemented".to_string());
        connection.write_frame(&response).await.unwrap();
    }
}


