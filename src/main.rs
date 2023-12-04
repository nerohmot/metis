
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

enum RecordType {
    ATR,
    FTR,
    Unknown,
}

fn determine_type(buf: &Vec<u8>) -> RecordType{
    match (buf[0], buf[1]){
        (0, 10) => RecordType::ATR,
        _ => RecordType::Unknown,
    } 
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
    /// returns the number of bytes available *AFTER** file_pointer.
    /// panics if the file is shorter than file_pointer.
    /// 
    fn bytes_available(&mut self) -> u64 {
        let bytes_in_file = self.file.seek(SeekFrom::End(0)).unwrap();
        if bytes_in_file < self.file_pointer {
            panic!("file '{}' turncated mid process!", self.file_path.file_name().unwrap().to_string_lossy())
        }
        if self.file.seek(SeekFrom::Start(self.file_pointer)).is_err(){

        };
        (bytes_in_file - self.file_pointer) as u64
    }

    ///
    /// returns true if there is a complete record available from file_pointer, false otherwhise.
    /// panics if the record length couldn't be read.
    /// 
    fn has_next_record(&mut self) -> bool {
        let bytes_available = self.bytes_available();
        if bytes_available >= 2 { // rec_len portion of the header is available
            let mut rec_len = [0u8;2];
            // self.file.seek(SeekFrom::Start(self.file_pointer));
            if self.file.read_exact(&mut rec_len).is_err(){
                panic!("couldn't read REC_LEN from '{}@{}' eventhough 2 bytes are available!", self.file_path.file_name().unwrap().to_string_lossy(), self.file_pointer);
            }
            let tail_length = match self.file_endian {
                Endian::Big => u16::from_be_bytes(rec_len),
                Endian::Little => u16::from_le_bytes(rec_len),
            };
            if bytes_available >= (4 + tail_length) as u64 {
                return true
            }
            return false
        } 
        false
    }

    ///
    /// returns a vector containing the full record (without the 2 lengthbytes)
    /// if no next record is available, it returns an empty vector
    /// 
    fn next_record(&mut self) -> Vec<u8> {
        if self.has_next_record(){
            let mut rec_len = [0u8;2]; 
            self.file.read_exact(&mut rec_len).unwrap();

            let tail_length = match self.file_endian {
                Endian::Big => u16::from_be_bytes(rec_len),
                Endian::Little => u16::from_le_bytes(rec_len),
            };
            let mut vec = vec![0u8; (tail_length + 2)  as usize]; // REC_TYP & REC_SUB added
            let bytes_read = self.file.read(vec.as_mut_slice()).unwrap();
            if bytes_read != (tail_length+2) as usize {
                panic!("WTF?")
            }
            self.file_pointer += (tail_length + 2) as u64;
            return vec;
        }
        Vec::new()
    }




    ///
    /// returns true if there is at least a record header to be read
    /// 
    fn peek_next_record(&mut self) -> bool {
        let bytes_available = self.bytes_available();
        if bytes_available >= 2 { // rec_len portion of the header is available
            // seek to file_pointer
            self.file.seek(SeekFrom::Start(self.file_pointer)).unwrap();
            // read rec_len_as_bytes
            let mut rec_len_bytes = [0u8;2]; 
            if self.file.read_exact(&mut rec_len_bytes).is_err(){
                return false
            }
            let mut tail_length:usize = 0;
            if cfg!(target_endian = "big") {
                tail_length = u16::from_be_bytes(rec_len_bytes) as usize;
            } else {
                tail_length = u16::from_le_bytes(rec_len_bytes) as usize;
            }
            let mut vec = vec![0u8; tail_length + 2  as usize]; // REC_TYP & REC_SUB added
            let bytes_read = self.file.read(vec.as_mut_slice()).unwrap();
            println!("read = {} bytes", bytes_read);
            println!("vec = {:?} {}", vec, vec.len());
            println!("{}", self.file.stream_position().unwrap());


            // let mut bytes = Vec::with_capacity(10).as_mut_slice();
            // let mut bytes = vec![0u8; 20];
            // if self.file.read_exact(bytes).is_err() {
            //     println!("error");
            // }
            // println!("{:?}", bytes);

            // let mut tial = vec![0u8; tail_length as usize];
            // let a = self.file.read_exact(&mut tail);
            // println!("{:?}", tail);
            
            



            return true
        }
        false
    }


    // fn get_next_record(&mut self) -> Vec<u8> {
    //     let bytes_available = self.bytes_available();
    //     if bytes_available >= 4 { // header is there
    //         // seek to file_pointer
    //         self.file.seek(SeekFrom::Start(self.file_pointer)).unwrap();
    //         // read header
    //         let mut rec_len = [0u8;2]; 
    //         if self.file.read_exact(&mut rec_len).is_err(){
    //             return false
    //         }
    //         // get the tail length
    //         let mut tail_length:usize = 0;
    //         if cfg!(target_endian = "big") {
    //             tail_length = u16::from_be_bytes(rec_len) as usize;
    //         } else {
    //             tail_length = u16::from_le_bytes(rec_len) as usize;
    //         }
    //         // create a buffer for the record tail
    //         let mut vec = vec![0u8; tail_length+4 as usize];
    //         let count = self.file.read(vec.as_mut_slice()).unwrap();
    //         println!("read = {} bytes", count);
    //         println!("vec = {:?}", vec);
    //         println!("{}", self.file.stream_position().unwrap());


    //         // let mut bytes = Vec::with_capacity(10).as_mut_slice();
    //         // let mut bytes = vec![0u8; 20];
    //         // if self.file.read_exact(bytes).is_err() {
    //         //     println!("error");
    //         // }
    //         // println!("{:?}", bytes);

    //         // let mut tial = vec![0u8; tail_length as usize];
    //         // let a = self.file.read_exact(&mut tail);
    //         // println!("{:?}", tail);
            
            



    //         return true
    //     }
    //     false
    // }




}

impl std::fmt::Display for STDReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_path.to_string_lossy())
    }
}


fn main() -> Result<(), std::io::Error> {
    let mut stdr = STDReader::new("v93k.STD".to_string(), true)?;
    for i in 1..3 {
        let rec = stdr.next_record();
        println!("{} : {:?}", i, rec.len())
    }
    Ok(())
}
