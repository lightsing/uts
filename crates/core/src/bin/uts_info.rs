// Copyright (C) The OpenTimestamps developers
// Copyright (C) The ots-rs developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # OpenTimestamps Viewer
//!
//! Simple application to open an OTS info file and dump its contents
//! to stdout in a human-readable format

use std::{
    env, fs, io,
    io::{BufReader, Seek},
    process,
};
use uts_core::codec::{
    Decode, Reader, VersionedProof,
    v1::{DetachedTimestamp, Timestamp},
};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <timestamp.ots>", args[0]);
        process::exit(1);
    }

    let mut fh = match fs::File::open(&args[1]) {
        Ok(fh) => BufReader::new(fh),
        Err(e) => {
            println!("Failed to open {}: {}", args[1], e);
            process::exit(1);
        }
    };

    match VersionedProof::<DetachedTimestamp>::decode(&mut Reader(&mut fh)) {
        Ok(ots) => {
            println!("OTS Detached Timestamp found:");
            println!("{ots}");
        }
        Err(e) => {
            println!(
                "Not a valid Detached Timestamp OTS file (trying raw timestamp): {}\n",
                e
            );
        }
    };

    fh.seek(io::SeekFrom::Start(0)).unwrap();

    match Timestamp::decode(&mut Reader(&mut fh)) {
        Ok(ots) => {
            println!("Raw Timestamp found:");
            println!("{ots}");
        }
        Err(e) => {
            println!("Failed to parse {}: {}", args[1], e);
            process::exit(1);
        }
    }
}
