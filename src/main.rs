pub mod util;
pub mod java_random;
pub mod java_random_crack;
pub mod random_string_utils;
pub mod random_string_utils_crack;

use std::time::{SystemTime, UNIX_EPOCH};
use clap::{Parser, Subcommand};
use random_string_utils::RandomStringUtils;
use random_string_utils::reverse_string;
use random_string_utils_crack::recover_seed;
use java_random::JavaRandom;
use java_random_crack::recover_seed as java_recover_seed;
use util::rollback_seed;

#[derive(Parser)]
#[command(arg_required_else_help(true))]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(Subcommand)]
enum Commands {
    RandomAlphanumeric {
        /// Output of RandomStringUtils.randomAlphanumeric(n)
        token: String,

        /// Number of tokens to output
        #[arg(short, long, default_value_t = 10)]
        count: u16,

        /// Length of tokens to output
        #[arg(short, long, default_value_t = 0)]
        output_len: u16,

        /// reverse tokens (support for older RandomStringUtils)
        #[arg(long, default_value_t = false)]
        old: bool,

        /// previous tokens to generate (newest to oldest)
        #[arg(long, short, default_value_t = 0)]
        previous: u128,
    },
    RandomAlphabetic {
        /// Output of RandomStringUtils.randomAlphabetic(n)
        token: String,

        /// Number of tokens to output
        #[arg(short, long, default_value_t = 10)]
        count: u16,

        /// Length of tokens to output
        #[arg(short, long, default_value_t = 0)]
        output_len: u16,

        /// reverse tokens (support for older RandomStringUtils)
        #[arg(long, default_value_t = false)]
        old: bool,

        /// previous tokens to generate (newest to oldest)
        #[arg(long, short, default_value_t = 0)]
        previous: u128,
    },
    NextInt {
        /// Outputs of random.nextInt(n)
        #[arg(use_value_delimiter = true, value_delimiter = ' ')]
        outputs: Vec<u128>,

        /// Value of the bound n
        #[arg(short, long)]
        n: u128,

        /// Number of values to output
        #[arg(short, long, default_value_t = 10)]
        count: u16,
    }
}

fn crack_randomstring(token: &String, count: u16, output_len: u16, old: bool, alphanumeric: bool, mut previous: u128) {
    let actual_token = if old {
        reverse_string(&token)
    } else {
        token.to_string()
    };
    let token_len = actual_token.len();
    if token_len < 9 {
        eprintln!("[!] Token length is {}, but a token of at least 9 characters is needed. Results may be incorrect.", token_len);
    }

    let outputs = actual_token.bytes().map(|b| (b - 32) as u128).collect::<Vec<u128>>();
    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let seed = recover_seed(outputs, alphanumeric, old);
    let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    eprintln!("\n");
    eprintln!("[*] Finished running after {}s", ((end - start) as f64)/1000.0);

    if seed.is_none() {
        eprintln!("[-] Could not recover seed!");
    } else {
        eprintln!("[+] Java Random seed recovered: {}", seed.unwrap());
        let l = if output_len == 0 { token_len } else { output_len as usize };
        let mut rsu = RandomStringUtils::new(seed.unwrap(), old);
        let mut delta = 1;
        let mut rolled_seed = rsu.random.seed;
        let mut check_token = token.to_string();
        let mut new_check_token = String::from("");
        let mut new_token;
        if previous > 1 {
            eprintln!("[+] The previous {} tokens were:", previous);
        } else if previous == 1 {
            eprintln!("[+] The previous token was:");
        }
        while previous > 0 {
            new_token = String::from("");
            while check_token != new_token {
                let mut rolling_rsu = RandomStringUtils::new_raw(rolled_seed, old);
                rolled_seed = rollback_seed(rolled_seed);
                new_check_token = if alphanumeric {
                    rolling_rsu.random_alphanumeric(token_len)
                } else {
                    rolling_rsu.random_alphabetic(token_len)
                };
                new_token = if alphanumeric {
                    rolling_rsu.random_alphanumeric(token_len)
                } else {
                    rolling_rsu.random_alphabetic(token_len)
                };
            }
            check_token = new_check_token.clone();
            eprintln!("[Δ-{}] {}", delta, check_token);
            previous -= 1;
            delta += 1;
        }
        if alphanumeric {
            rsu.random_alphanumeric(token_len);
        } else {
            rsu.random_alphabetic(token_len);
        }
        delta = 1;
        if count == 1 {
            eprintln!("[+] The next token is:");
        } else {
            eprintln!("[+] The next {} tokens are:", count);
        }
        for _ in 0..count {
            println!("[Δ+{}] {}", delta, if alphanumeric {
                rsu.random_alphanumeric(l)
            } else {
                rsu.random_alphabetic(l)
            });
            delta += 1;
        }
    }
}

fn main() {
    let args = Args::parse();
    match &args.command {
        Some(Commands::RandomAlphanumeric { token, count, output_len, old, previous }) => {
            crack_randomstring(token, *count, *output_len, *old, true, *previous);
        },
        Some(Commands::RandomAlphabetic { token, count, output_len, old, previous}) => {
            crack_randomstring(token, *count, *output_len, *old, false, *previous);
        },
        Some(Commands::NextInt { outputs, n, count }) => {
            let need = (48. / (*n as f32).log2()).ceil() as usize;
            if outputs.len() < need {
                eprintln!("[!] Number of outputs is {}, but at least {} are needed. Results may be incorrect", outputs.len(), need);
            }

            if *n <= 6 {
                eprintln!("[!] Bound is very small, recovery may not work.");
            }

            let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
            let seed = java_recover_seed(outputs.clone(), *n);
            let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
            eprintln!("\n");
            eprintln!("[*] Finished running after {}s", ((end - start) as f64)/1000.0);

            if seed.is_none() {
                eprintln!("[-] Could not recover seed!");
            } else {
                eprintln!("[+] Java Random seed recovered: {}", seed.unwrap());
                let mut rand = JavaRandom::new(seed.unwrap());
                for _ in 0..outputs.len() {
                    rand.next_int(*n);
                }
                if *count == 1 {
                    eprintln!("[+] The next output is:");
                } else {
                    eprintln!("[+] The next {} outputs are:", count);
                }
                for _ in 0..*count {
                    println!("{}", rand.next_int(*n));
                }
            }
        },
        None => {}
    }
}
