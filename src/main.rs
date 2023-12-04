
use std::convert::TryInto;
use std::os::unix::fs::FileExt;
// use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::fs::File;
// use std::io;
use std::io::prelude::*;
use std::io::{SeekFrom, ErrorKind, Error};
// extern crate byte;
// use byte::ctx::Endian;

#[derive(Debug)]
enum Endian {
    Big,
    Little,
}

fn determine_endian(buf: &[u8;2], expected_value: u16) -> Endian {
    if cfg!(target_endian = "big") {
        let be_conversion = u16::from_be_bytes(*buf);
        if be_conversion == expected_value {
            return Endian::Big;
        } else {
            return Endian::Little;
        }
    } else {
        let le_conversion = u16::from_le_bytes(*buf);
        if le_conversion == expected_value {
            return Endian::Little;
        } else {
            return Endian::Big;
        }
    }
}


struct STDReader {
    file_path : PathBuf,
    file : std::fs::File,
    file_endian : Endian,
    file_pointer : u64,
}

impl STDReader {
    fn new(file_name:String, file_name_compliance:bool)-> Result<STDReader, Error> {
        // derive the absolute path
        let file_path = match Path::new(&file_name).canonicalize() {
            Ok(file_path) => file_path,
            Err(e) => return Err(e),
        };
        // in case of desired file-name compliance (spec page 65) ...
        if file_name_compliance {
            // get the extension and verify there is one
            let file_extension = match file_path.extension(){
                Some(file_extension) => file_extension.to_os_string().into_string().unwrap(),
                None => return Err(Error::new(ErrorKind::Other, "file has no file extension")),
            };
            // verify the file extension starts with "STD"
            if !file_extension.starts_with("STD"){
                return Err(Error::new(ErrorKind::Other, "file has a file extension not compliant to the STDF spec"))
            }
        }
        // Open file for reading
        let mut file = match File::options()
            .read(true)
            .write(false)
            .open(&file_path){
                Ok(file) => file,
                Err(e) => return Err(e),
        };
        // Get the file length and verify there is at least 6 bytes present (FAR is first record and holds 6 bytes)
        let file_length = file.seek(SeekFrom::End(0))?;
        if file_length < 6u64 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "File is too short (maybe not an STDF file?)"));
        }
        // Read the FAR and determine the endian.
        let mut buf = [0u8;6]; // FAR is located at the first 6 bytes of the file
        if file.read_exact_at(&mut buf, 0).is_err(){
            return Err(Error::new(ErrorKind::Other, "File has 6 bytes, but couldn't be read"))
        }
        let endian = determine_endian(buf[0..2].try_into().unwrap(), 2);

        Ok(STDReader {
            file_path : file_path,
            file : file,
            file_pointer: 0,
            file_endian : endian,
        })
    }


    /// 
    /// Given two bytes and the expected value, returns the endian.
    /// 

    //
    fn read_record_header(&self) -> Vec<u8> {
        let retval = Vec::new();
        // if file.read_exact_at(&mut buf, 0).is_err(){
        //     println!("Error")
        // }
        // println!("{:?}", &buf);
        retval   
    }

    // This method returns true if there is a next complete record to be read
    fn has_next(&self) -> bool{
        true
    }

    // This method returns true if there is a partial record to be read
    fn has_partial_next(&self) -> bool{
        true
    }

    // This method returns the next complete record in the form of a vector
    // without moving the file_pointer
    fn peek_next(&self) -> Vec<u8> {
        Vec::new()
    }

    ///
    /// returns ture if there is at least a record header to be read
    /// 
    fn has_next_header(&self) -> bool {
        true
    }

    ///
    /// returns how many bytes are ready to be read
    ///
    fn bytes_available(&mut self) -> u64 {
        self.file.seek(SeekFrom::End(0)).unwrap() - self.file_pointer
    }



}

impl std::fmt::Display for STDReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_path.to_string_lossy())
    }
}


fn main() -> Result<(), std::io::Error> {
    let mut stdr = STDReader::new("v93k.STD".to_string(), true)?;
    let b = stdr.bytes_available();

    println!("{}", b);
    Ok(())
}
