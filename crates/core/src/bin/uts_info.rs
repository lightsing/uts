// Copyright (C) The OpenTimestamps developers
// Copyright (C) The ots-rs developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # OpenTimestamps Viewer
//!
//! Simple application to open an OTS info file and dump its contents
//! to stdout in a human-readable format

use std::{env, fs, io::BufReader, process};
use uts_core::codec::{Decode, VersionedProof, v1::Timestamp};

fn main() {
    // env_logger::init();
    //
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <timestamp.ots>", args[0]);
        process::exit(1);
    }

    let fh = match fs::File::open(&args[1]) {
        Ok(fh) => BufReader::new(fh),
        Err(e) => {
            println!("Failed to open {}: {}", args[1], e);
            process::exit(1);
        }
    };

    let ots = match VersionedProof::<Timestamp>::decode(fh) {
        Ok(ots) => ots,
        Err(e) => {
            println!("Failed to parse {}: {}", args[1], e);
            process::exit(1);
        }
    };

    println!("{}", ots);
}
