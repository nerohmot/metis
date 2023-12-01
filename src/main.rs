use std::os::unix::fs::FileExt;
// use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::fs::File;
// use std::io;
use std::io::prelude::*;
use std::io::{SeekFrom, ErrorKind, Error};

struct STDReader {
    file_path : PathBuf,
    file : std::fs::File,
    // endian : 
    file_pointer : usize,
}

impl STDReader {
    fn new(file_name:String, file_name_compliance:bool)-> Result<STDReader, Error> {
        // derive the absolute path
        let file_path = match Path::new(&file_name).canonicalize() {
            Ok(file_path) => file_path,
            Err(e) => return Err(e),
        };
        // in case of desired file name compliance (spec page 65) ...
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
        let x = file.read_exact_at(&mut buf, 0);
        println!("{:?} : {:?}", x, &buf);





        Ok(STDReader {
            file_path : file_path,
            file : file,
            file_pointer:0,
        })
    }

    fn has_next(&self) -> bool{
        true
    }

    fn has_partial_next(&self) -> bool{
        true
    }

    fn peek_next(&self) -> Vec<u8> {
        Vec::new()
    }





}

impl std::fmt::Display for STDReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_path.to_string_lossy())
    }
}


fn main() -> Result<(), std::io::Error> {
    let stdr = STDReader::new("v93k.STD".to_string(), true)?;
    println!("{}", stdr);
    Ok(())
}
