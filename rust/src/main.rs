use std::{env, fs, process};
use std::io::Write;
use turb1600::turb1600_hash;


/// Print bytes in hex
fn print_hex(bytes: &[u8]) {
    for b in bytes {
        print!("{:02x}", b);
    }
    println!();
}

/// Print bytes as UTF-8 if possible, else fallback to hex
fn print_utf8_or_hex(bytes: &[u8]) {
    match std::str::from_utf8(bytes) {
        Ok(s) => println!("{}", s),
        Err(_) => print_hex(bytes),
    }
}

/// Show usage and exit
fn usage() -> ! {
    eprintln!(
        "Usage:
  turb1600 <string>                 Hash a string
  turb1600 --hex <hex-string>       Hash raw bytes from hex
  turb1600 --file <path>            Hash file contents
  turb1600 --tag <tag> <string>     Hash string with domain tag
Options:
  --raw                              Output raw bytes instead of hex"
    );
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        usage();
    }

    let mut raw_output = false;
    let mut arg_start = 1;

    // Check for --raw
    if args[1] == "--raw" {
        raw_output = true;
        arg_start += 1;
        if args.len() <= arg_start {
            usage();
        }
    }

    let input: Vec<u8> = match args[arg_start].as_str() {
        "--hex" => {
            if args.len() <= arg_start + 1 {
                usage();
            }
            hex::decode(&args[arg_start + 1]).expect("Invalid hex input")
        }

        "--file" => {
            if args.len() <= arg_start + 1 {
                usage();
            }
            fs::read(&args[arg_start + 1]).expect("Failed to read file")
        }

        "--tag" => {
            if args.len() <= arg_start + 2 {
                usage();
            }
            let mut v = Vec::new();
            v.extend_from_slice(args[arg_start + 1].as_bytes());
            v.push(0x00); // domain separator
            v.extend_from_slice(args[arg_start + 2].as_bytes());
            v
        }

        _ => args[arg_start].as_bytes().to_vec(),
    };

    let out = turb1600_hash(&input);

    if raw_output {
        // print raw bytes to stdout
        std::io::stdout().write_all(&out).expect("Failed to write output");
    } else {
        print_hex(&out);
    }
}
