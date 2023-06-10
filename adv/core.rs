use crate::encryption;
use crate::models;
use crate::utils;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Error;
use std::str;
#[cfg(not(target_arch = "wasm32"))]
use std::ffi::OsStr;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::metadata;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc;
#[cfg(not(target_arch = "wasm32"))]
use std::thread;
#[cfg(not(target_arch = "wasm32"))]
use tar::Archive;
#[cfg(not(target_arch = "wasm32"))]
use threadpool::ThreadPool;

#[cfg(not(target_arch = "wasm32"))]
pub fn create_tar_gz(folder_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut fname = folder_name.to_string();
    fname.push_str(".tar.wdc");
    let mut tar = tar::Builder::new(File::create(&fname)?);
    println!("Tarring {} to {}", folder_name, &fname);
    tar.append_dir_all(folder_name, folder_name)?;
    fs::remove_dir_all(folder_name)?;
    println!("Tarred {} to {}", &folder_name, &fname);
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn tar_all_dirs() -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(".")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    let (tx, rx) = mpsc::channel();
    for path in entries {
        if metadata(&path)?.is_dir() {
            let pth = path.display().to_string();
            let tx = tx.clone();
            let folder = models::Folder::new(&pth);
            folder.tar().unwrap();
            tx.send(0).unwrap();
        }
    }
    drop(tx);
    for _ in rx {}
    Ok(())
}

pub fn encrypt_file(fname: &str, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let in_file = BufReader::new(File::open(fname)?);
    let out_file = BufWriter::new(File::create(encryption::encrypt_bytes_to_string(
        key,
        fname.as_bytes(),
    ))?);
    println!("Encrypting {}", fname);
    encryption::encrypt_file_to_file_buffered(key, in_file, out_file)?;
    println!("Encrypted {}", fname);
    fs::remove_file(fname)?;
    Ok(())
}

pub fn decrypt_file(mut fname: &str, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let in_file = BufReader::new(File::open(fname)?);
    fname = &fname[2..];
    let decrypted_file_name = encryption::decrypt_from_string(key, fname)?;
    if !utils::warn_if_file_exists(&decrypted_file_name) {
        return Ok(());
    }
    let out_file = BufWriter::new(File::create(&decrypted_file_name)?);
    println!("Decrypting {}", decrypted_file_name);
    encryption::decrypt_file_to_file_buffered(key, in_file, out_file)?;
    println!("Decrypted {}", decrypted_file_name);
    fs::remove_file(fname)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn encrypt_all_files(fernet_key: &'static str) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();
    let pool = ThreadPool::new(num_cpus::get());
    let entries = fs::read_dir(".")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    for path in entries {
        let tx = tx.clone();
        let file_name = path.display().to_string();
        if path.metadata()?.is_dir() {
            println!("{} is a directory!", file_name);
            continue;
        }
        if file_name != "./.secret.key" {
            pool.execute(move || {
                let file = models::File::new(&file_name, fernet_key);
                file.encrypt().unwrap();
                tx.send(1).unwrap();
            });
        }
    }
    drop(tx);
    for _ in rx {}
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn decrypt_all_files(fernet_key: &'static str) -> Result<(), Error> {
    let (tx, rx) = mpsc::channel();
    let pool = ThreadPool::new(num_cpus::get());
    let entries = fs::read_dir(".")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    for path in entries {
        let tx = tx.clone();
        let file_name = path.display().to_string();
        if path.metadata()?.is_dir() {
            println!("{} is a directory!", file_name);
            continue;
        }
        if file_name != "./.secret.key" {
            pool.execute(move || {
                let file = models::File::new(&file_name, fernet_key);
                if file.decrypt().is_err() {
                    println!("Cannot decrypt file {}", file_name)
                }
                tx.send(1).unwrap();
            });
        }
    }
    drop(tx);
    for _ in rx {}
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn untar_dir(dir_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", dir_name);
    let file = File::open(dir_name)?;
    let mut archive = Archive::new(file);
    println!("Untarring {}", dir_name);
    archive.unpack(".")?;
    println!("Untarred {}", dir_name);
    fs::remove_file(dir_name)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn untar_all_dirs() -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(".")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    let (tx, rx) = mpsc::channel();
    for path in entries {
        if let Some("wdc") = path.extension().and_then(OsStr::to_str) {
            let pth = path.display().to_string();
            let tx = tx.clone();
            thread::spawn(move || {
                let folder = models::Folder::new(&pth);
                folder.untar().unwrap();
                tx.send(0).unwrap();
            });
        }
    }
    drop(tx);
    for _ in rx {}
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn untar_all_dirs() -> Result<(), Box<dyn std::error::Error>> {
    println!("Untarring not supported in wasm32");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn tar_all_dirs() -> Result<(), Box<dyn std::error::Error>> {
    println!("Tarring not supported in wasm32");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn encrypt_all_files(fernet_key: &'static str) -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(".")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    for path in entries {
        let file_name = path.display().to_string();
        if path.metadata()?.is_dir() {
            println!("{} is a directory!", file_name);
            continue;
        }
        if file_name != "./.secret.key" {
            let file = models::File::new(&file_name, fernet_key);
            file.encrypt().unwrap();
        }
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn decrypt_all_files(fernet_key: &'static str) -> Result<(), Error> {
    let entries = fs::read_dir(".")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    for path in entries {
        let file_name = path.display().to_string();
        if path.metadata()?.is_dir() {
            println!("{} is a directory!", file_name);
            continue;
        }
        if file_name != "./.secret.key" {
            let file = models::File::new(&file_name, fernet_key);
            if file.decrypt().is_err() {
                println!("Cannot decrypt file {}", file_name)
            }
        }
    }
    Ok(())
}