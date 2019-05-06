#[macro_use]
extern crate arrayref;

pub mod aes;
mod constants;

fn main() {
    let message_text = "Hello, world! <3";
    let mut message_array = [0; 16];
    message_array.copy_from_slice(message_text.as_bytes());

    let key_text = "I like computers";
    let mut key_array = [0; 16];
    key_array.copy_from_slice(key_text.as_bytes());

    let block = aes::Block::new(&message_array);
    let key = aes::Key::new(&key_array);

    let encrypted = aes::encrypt(key, block);
    let decrypted = aes::decrypt(key, encrypted);

    println!("Message:   {}", message_text);
    println!("Key:       {}", key_text);
    println!("Encrypted: {}", String::from_utf8_lossy(&encrypted.as_bytes()[..]));
    println!("Decrypted: {}", String::from_utf8_lossy(&decrypted.as_bytes()[..]));
}
