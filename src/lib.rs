/*
    tatar: a tar implementation in Rust.
    Copyright 2016 Sam Saint-Pettersen.

    Dual licensed under the GPL and MIT licenses;
    see GPL-LICENSE and MIT-LICENSE respectively.
*/

extern crate regex;
extern crate filetime;
use std::char;
use std::fs;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::fs::File;
use self::regex::Regex;
use self::filetime::FileTime;

static EOF_PADDING: usize = 512;

struct TarEntry {
    part: String,
    file: String,
}

pub struct Tatar;

impl Tatar {
    fn dec_to_padded_octal(num: u64, length: usize) -> String {
        let octal = format!("{:o}", num);
        let mut padding = String::new();
        for _ in 0 .. length - octal.len() {
            padding = format!("{}{}", padding, '0');
        }
        format!("{}{}", padding, octal)
    }

    fn pad_data(length: usize) -> String {
        let mut padding = String::new();
        for _ in 1 .. length {
            padding = format!("{}{}", padding, char::from_u32(0).unwrap());
        }
        padding
    }

    fn write_padded_data(data: &str) -> String {
        let mut eof: usize = EOF_PADDING;
        let mut m: usize = 1;
        while eof < data.len() {
            eof = eof * m;
            if data.len() <= eof {
                break;
            }
            m += 1;
        }
        format!("{}{}", data, Tatar::pad_data(eof - (data.len() - 1)))
    }

    fn calc_checksum(header: String) -> String {
        let mut checksum = 0;
        for h in header.chars() {
            checksum += h as u64;
        }
        Tatar::dec_to_padded_octal(checksum - 64, 6)
    }

    fn write_tar_entry(tarname: &str, filename: &str) -> String {
        let metadata = fs::metadata(filename).unwrap();
        let mut input = File::open(filename).unwrap();
        let mut contents = String::new();
        let _ = input.read_to_string(&mut contents);
        let size = Tatar::dec_to_padded_octal(metadata.len(), 11);
        let mtime = FileTime::from_last_modification_time(&metadata);
        let modified = Tatar::dec_to_padded_octal(mtime.seconds_relative_to_1970(), 11);
        let etype = 0;
        let nc = char::from_u32(0).unwrap();
        /*
          * TAR FORMAT SPECIFICATION
          * (a) File name (0-)
          * (b) File mode (100; 8)
          * (c) Owner's numeric user ID (108; 8)
          * (d) Group's numeric user ID (116; 8)
          * (e) File size in bytes (octal) (124; 12)
          * (f) Last modification time in numeric Unix time format (octal) (136; 12)
          * (g) Checksum for header record (148; 8)
          * (h) Link indicator (file type) (156; 1)
          * (i) UStar indicator (257; 6)
        */
        let mut tar = File::create(format!("_{}_", tarname)).unwrap();
        let mut header = format!("{}{}", filename, Tatar::pad_data(101 - filename.len()));
        header = format!("{}0100777{}0000000{}0000000{}{}{}{}{}", 
        header, nc, nc, nc, size, nc, modified, nc);
        header = format!("{}000000{} {}{}ustar{}00{}", 
        header, nc, etype, Tatar::pad_data(101), nc, Tatar::pad_data(248));
        let data = Tatar::write_padded_data(&contents);
        let hd = format!("{}{}", header, data);
        let _ = tar.write_all(hd.as_bytes());
        header
    }

    fn write_tar_entries(tarname: &str, entries: Vec<TarEntry>) {
        let mut e: Vec<String> = Vec::new();
        for i in 0 .. entries.len() {
            let header = Tatar::write_tar_entry(&entries[i].part, &entries[i].file);
            e.push(Tatar::write_checksum(&entries[i].part, header));
        }
        Tatar::finalize_tar(&Tatar::merge_entries(&tarname, e), tarname);
    }

    fn write_checksum(tarname: &str, header: String) -> String {
        let t1 = format!("_{}_", tarname);
        let t2 = format!("__{}__", tarname);
        let mut input = File::open(t1.clone()).unwrap();
        let mut contents = String::new();
        let _ = input.read_to_string(&mut contents);
        let mut tar = File::create(t2.clone()).unwrap();
        let _ = tar.write_all(contents.as_bytes());
        let _ = tar.seek(SeekFrom::Start(148));
        let _ = tar.write_all(Tatar::calc_checksum(header).as_bytes());
        let _ = fs::remove_file(t1).unwrap();
        t2.clone()
    }

    fn merge_entries(tarname: &str, entries: Vec<String>) -> String {
        let mut contents = String::new();
        for entry in entries {
            let mut input = File::open(entry.clone()).unwrap();
            let _ = input.read_to_string(&mut contents);
            let _ = fs::remove_file(entry.clone()).unwrap();
        }
        let temp = format!("___{}___", tarname);
        let mut tar = File::create(temp.clone()).unwrap();
        let _ = tar.write_all(contents.as_bytes());
        temp
    }

    fn finalize_tar(temp: &str, tarname: &str) {
        let mut input = File::open(temp).unwrap();
        let mut contents = String::new();
        let _ = input.read_to_string(&mut contents);
        let mut tar = File::create(tarname).unwrap();
        let _ = tar.write_all(format!("{}{}", 
        contents, Tatar::pad_data((EOF_PADDING * 2) + 1)).as_bytes());
        fs::remove_file(temp).unwrap();
    }

    pub fn create_single_tar(tarname: &str, filename: &str) {
        let header = Tatar::write_tar_entry(tarname, filename);
        Tatar::finalize_tar(&Tatar::write_checksum(tarname, header), tarname);
    }

    pub fn create_multi_tar(tarname: &str, filenames: Vec<&str>) {
        let mut entries: Vec<TarEntry> = Vec::new();
        for i in 0 .. filenames.len() {
            if filenames[i].len() < 100 {
                let r = Regex::new(r"\.tar$").unwrap();
                let repl = format!(".{}", i);
                let pn = r.replace_all(&tarname, &repl[..]);
                entries.push(TarEntry { part: pn, file: filenames[i].to_owned() });
            }
        }
        Tatar::write_tar_entries(tarname, entries);
    }
}
