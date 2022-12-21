extern crate low;
extern crate walkdir;
use std::io::prelude::*;
use std::io::BufWriter;
use std::env;
use std::fs;
use walkdir::{DirEntry, WalkDir};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::mem::transmute;
use std::fs::metadata;

fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
         .file_name()
         .to_str()
         .map(|s| entry.depth() == 0 || !s.starts_with("."))
         .unwrap_or(false)
}

fn main() -> Result<(), std::io::Error > {
	let enableCompression = false;
    let args: Vec<String> = env::args().collect();
    let rootDir: String;
    if args.len() > 1 {
        rootDir = String::from( &args[ 1 ] );
    } else {
        rootDir = String::from( env::current_dir()?.as_path().to_string_lossy() );
    }
    let targetFilename: String; 
    if  args.len() > 2 {
        targetFilename = args[ 2 ].clone();
    } else {
        targetFilename = "website.pack".to_string();
    }

    let walkDir =  WalkDir::new(rootDir.as_str() )
            .into_iter()
            .filter_entry(|e| is_not_hidden(e))
            .filter_map(|v| v.ok() );

    let output = fs::File::create( targetFilename )?;
    let mut stream = BufWriter::new( output );
    for dir in walkDir {
        let mut webPath : String = dir.path().to_str().unwrap().to_string();
        webPath = webPath.replace( rootDir.as_str(), "" );
        webPath = webPath.replace( '\\',  "/" );
		webPath = webPath.replace( "/index.html",  "/" );
        let md = metadata(dir.path()).unwrap();
        if ( webPath.is_empty() || md.is_dir() ) {
            continue;
        }
		let mimeType : &str;
		let contentType: String = match dir.path().extension() {
			Some( ext ) => { 
				mimeType = match ext.to_ascii_lowercase().to_str().unwrap() {
					"css" => { "text/css" }
					"js" => { "text/javascript" }
					"mp3" => { "audio/*" }
					"ogg" => { "audio/ogg" }
					"gif" => { "image/gif" }
					"bmp" => { "image/bmp" }
					"png" => { "image/png" }
					"jpg" | "jpeg" => { "image/jpeg" }
					"html" => { "text/html; charset=utf-8" }
					_ => { "application/octet-stream" }
				};
				format!( "Content-Type: {} \r\n", mimeType )
			}
			None => { 
				mimeType = "text/plain";
				"Content-Type: text/plain\r\n".to_string() 
			}
		};
        let mut sizeBytes: [u8;4];
        unsafe {
            let len: u32 = webPath.as_bytes().len() as u32;
            sizeBytes = transmute( len.to_le() );
        }
        stream.write_all( &sizeBytes ) ?;
        stream.write_all( webPath.as_bytes() ) ?;

		if ( enableCompression ) {
			let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
			println!( "Compress {:^30} {}",  format!( "[{}] ", mimeType ).as_str(), webPath.as_str() );
			let input = fs::read( dir.path() )?;
			e.write_all(input.as_slice())?;
			let compressed_bytes = e.finish()?;
			let compressedLen: u32 = compressed_bytes.len() as u32;
			let anwser = format!( "HTTP/1.0 200 OK\r\nContent-Encoding: gzip\r\n{}Content-Length: {}\r\n\r\n", contentType, compressedLen );
			let anwserBuffer = anwser.as_bytes();
			unsafe {
				let len = compressedLen + ( anwserBuffer.len() as u32 );
				sizeBytes = transmute( len.to_le() );
			}
			stream.write_all( &sizeBytes ) ?;
			stream.write_all( anwserBuffer ) ?;
			stream.write_all( compressed_bytes.as_slice() ) ?;
		} else {
			println!( "Copy {:^30} {}",  format!( "[{}] ", mimeType ).as_str(), webPath.as_str() );
			let input = fs::read( dir.path() )?;
			let dataLen: u32 = input.len() as u32;
			let anwser = format!( "HTTP/1.0 200 OK\r\n{}Content-Length: {}\r\n\r\n", contentType, dataLen );
			let anwserBuffer = anwser.as_bytes();
			unsafe {
				let len = dataLen + ( anwserBuffer.len() as u32 );
				sizeBytes = transmute( len.to_le() );
			}
			stream.write_all( &sizeBytes ) ?;
			stream.write_all( anwserBuffer ) ?;
			stream.write_all( input.as_slice() ) ?;
		}
    }
    stream.flush();

    Ok(())
}
