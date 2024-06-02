use serde::{Serialize, Deserialize};
use serde_json::{to_string, from_str};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;


#[derive(Debug, Serialize, Deserialize)]
pub struct Protocol { 
    auth: String,   //Session. En caso de una desconexión podría recuperarse para seguir siendo el dueño de sus monedas.
                    //         Si el servidor recibe una nueva conexión sin auth genera una sesión y devuelve en este campo el identificador.
    message: String //Action, result, etc.
}

impl Protocol {
    pub fn new(auth: String, message: String) -> Protocol {
        Protocol { auth, message }
    }

    pub async fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let serialized_msg = to_string(self).unwrap();
        stream.write_all(serialized_msg.as_bytes()).await?;
        Ok(())

        
        //protocol.send(&mut stream).await?;
    }
    
    pub async fn receive(stream: &mut TcpStream) -> std::io::Result<Protocol> {
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await?;
        let serialized_msg = String::from_utf8(buffer).unwrap();
        let protocol: Protocol = from_str(&serialized_msg).unwrap();
        Ok(protocol)

        // let protocol = Protocol::receive(&mut server_stream).await?;
    }
}
