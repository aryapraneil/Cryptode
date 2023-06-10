const FERNET_FILE: &str = ".secret.key";

use crate::utils::ask_bool;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::process;
use std::str;

pub fn encrypt_bytes_to_string(key: &str, content: &[u8]) -> String {
    let fernet = fernet::Fernet::new(key).unwrap();
    fernet.encrypt(content)
}

pub fn encrypt_file_to_file_buffered(
    key: &str,
    mut reader: BufReader<File>,
    mut writer: BufWriter<File>,
) -> Result<(), Box<dyn std::error::Error>> {
    let fernet = fernet::Fernet::new(key).unwrap();
    let mut buffer = vec![0; 8192];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        writer.write_all(fernet.encrypt(&buffer[0..n]).as_bytes())?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;
    Ok(())
}

pub fn decrypt_from_string(
    key: &str,
    ciphertext: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let fernet = fernet::Fernet::new(key).unwrap();
    Ok(String::from_utf8(fernet.decrypt(ciphertext)?)?)
}

pub fn decrypt_file_to_file_buffered(
    key: &str,
    mut reader: BufReader<File>,
    mut writer: BufWriter<File>,
) -> Result<(), Box<dyn std::error::Error>> {
    let fernet = fernet::Fernet::new(key).unwrap();
    let mut buffer = String::new();
    loop {
        let n = reader.read_line(&mut buffer)?;
        if n == 0 {
            break;
        }
        buffer.pop();
        writer.write_all(&fernet.decrypt(&buffer[0..n - 1])?)?;
        buffer.clear();
    }
    writer.flush()?;
    Ok(())
}

pub fn write_fernet_key_to_file(key: String) -> &'static str {
    if Path::new(FERNET_FILE).exists() {
        println!("{} already exists", FERNET_FILE);
        if ask_bool("Do you want to use the existing key?", false).unwrap() {
            return read_fernet_key_from_file();
        }
        process::exit(1);
    }
    let mut file = File::create(FERNET_FILE).unwrap();
    file.write_all(key.as_bytes()).unwrap();
    Box::leak(key.into_boxed_str())
}

pub fn read_fernet_key_from_file() -> &'static str {
    if !Path::new(FERNET_FILE).exists() {
        println!("{} doesn't exist", FERNET_FILE);
        process::exit(1);
    }
    let mut file = File::open(FERNET_FILE).unwrap();
    let mut key = String::new();
    file.read_to_string(&mut key).unwrap();
    Box::leak(key.into_boxed_str())
}