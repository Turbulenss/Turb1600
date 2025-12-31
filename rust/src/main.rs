use std::{env, fs, process};
use turb1600::turb1600_hash;

fn print_hex(bytes: &[u8]) {
    for b in bytes {
        print!("{:02x}", b);
    }
    println!();
}

fn usage() -> ! {
    eprintln!(
        "Usage:
  turb1600 <string>
  turb1600 --hex <hex-string>
  turb1600 --file <path>
  turb1600 --tag <tag> <string>"
    );
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
    }

    let input: Vec<u8> = match args[1].as_str() {
        "--hex" => {
            if args.len() != 3 {
                usage();
            }
            hex::decode(&args[2]).expect("invalid hex")
        }

        "--file" => {
            if args.len() != 3 {
                usage();
            }
            fs::read(&args[2]).expect("failed to read file")
        }

        "--tag" => {
            if args.len() != 4 {
                usage();
            }
            let mut v = Vec::new();
            v.extend_from_slice(args[2].as_bytes());
            v.push(0x00); // domain separator
            v.extend_from_slice(args[3].as_bytes());
            v
        }

        _ => args[1].as_bytes().to_vec(),
    };

    let out = turb1600_hash(&input);
    print_hex(&out);
}
