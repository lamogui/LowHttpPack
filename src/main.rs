extern crate walkdir;
use std::io::prelude::*;
use std::io::BufWriter;
use std::env;
use std::fs;
use walkdir::{DirEntry, WalkDir};
use flate2::Compression;
use flate2::write::GzEncoder;
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
		let mut enableCompression = false;
		let contentType: String = match dir.path().extension() {
			Some( ext ) => { 
				mimeType = match ext.to_ascii_lowercase().to_str().unwrap() {
					"css" => { enableCompression = true; "text/css" }
					"js" => {  enableCompression = true;"text/javascript" }
					"mp3" => { "audio/*" }
					"ogg" => { "audio/ogg" }
					"gif" => { "image/gif" }
					"bmp" => { enableCompression = true; "image/bmp" }
					"png" => { "image/png" }
					"ico" => { "image/x-icon" }
					"txt"|"nfo" => { enableCompression = true; "text/plain" }
					"jpg" | "jpeg" => { "image/jpeg" }
					"html" => {  enableCompression = true; "text/html; charset=utf-8" }
					"rar"|"zip"|"gz"|"7z" => { "application/octet-stream" }
					_ => {  enableCompression = true; "application/octet-stream" }
				};
				format!( "Content-Type: {} \r\n", mimeType )
			}
			None => { 
				mimeType = "text/plain";
				"Content-Type: text/plain\r\n".to_string() 
			}
		};
		let webPathBytes = webPath.as_bytes();
		let webPathSize = webPathBytes.len() as u32;
        stream.write_all( &webPathSize.to_le_bytes() ) ?;
        stream.write_all( webPathBytes ) ?;

		if ( enableCompression ) {
			let mut e = GzEncoder::new(Vec::new(), Compression::best());
			println!( "Compress {:^30} {}",  format!( "[{}] ", mimeType ).as_str(), webPath.as_str() );
			let input = fs::read( dir.path() )?;
			e.write_all(input.as_slice())?;
			let compressed_bytes = e.finish()?;
			let compressedLen: u32 = compressed_bytes.len() as u32;
			let anwser = format!( "HTTP/1.0 200 OK\r\nContent-Encoding: gzip\r\n{}Content-Length: {}\r\n\r\n", contentType, compressedLen );
			let anwserBuffer = anwser.as_bytes();
			stream.write_all( &(anwserBuffer.len() as u32).to_le_bytes() ) ?;
			stream.write_all( anwserBuffer ) ?;
			stream.write_all( compressed_bytes.as_slice() ) ?;
		} else {
			println!( "Copy     {:^30} {}",  format!( "[{}] ", mimeType ).as_str(), webPath.as_str() );
			let input = fs::read( dir.path() )?;
			let dataLen: u32 = input.len() as u32;
			let anwser = format!( "HTTP/1.0 200 OK\r\n{}Content-Length: {}\r\n\r\n", contentType, dataLen );
			let anwserBuffer = anwser.as_bytes();
			stream.write_all( &(anwserBuffer.len() as u32).to_le_bytes() ) ?;
			stream.write_all( anwserBuffer ) ?;
			stream.write_all( input.as_slice() ) ?;
		}
    }
    stream.flush();

    Ok(())
}
