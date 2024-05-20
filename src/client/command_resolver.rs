use std::error::Error;

// por ahora en el cliente, despues va a pasar al servidor
mod machine;

pub fn insert_coin() -> Result<u32, Box<dyn Error>> {
    Ok(0)
}

pub fn consult_pool() -> Result<u32, Box<dyn Error>> {
    Ok(0)
}

pub fn leave() {
    println!("Desconectandose del servidor...");
}
