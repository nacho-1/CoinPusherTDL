use std::error::Error;

pub fn insert_coin() -> Result<u32, Box<dyn Error>> {
    Ok(0)
}

pub fn consult_pool() -> Result<u32, Box<dyn Error>> {
    Ok(0)
}

pub fn leave() {
    println!("Desconectandose del servidor...");
}
