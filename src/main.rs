use std::io::{self, Write};
use clap::Parser;
use colored::Colorize;
mod function;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

struct Args {
    #[arg(short, long)]
    install: Option<String>,

    #[arg(short, long)]
    sync: bool,
}

fn main() {
    let args = Args::parse(                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       );
    if let Some(pkgname) = args.install {
        let index = match function::read_index() {
            Ok(index) => index,
            Err(err) => {
                eprintln!("{}", err.red().bold());
                std::process::exit(1);
            }
        };
        let package = function::find_package(&pkgname, &index);
        match package {
            Some(pkg) => {
                let mut answer = String::new();

                println!(
                    "{} {}\n{} {}\n{} {}\n",
                    "Repository:".bold(),
                    index.repository,
                    "Package:".bold(),
                    pkg.name.cyan().bold(),
                    "Version:".bold(),
                    pkg.version.green()
                );

                print!("{}", "do you want to install this package? [y/n] > ".bright_white());
                io::stdout()
                    .flush()
                    .expect("cannot flush stdout");

                io::stdin()
                    .read_line(&mut answer)
                    .expect("input error");

                let answer = answer.trim();
                match answer {
                    "y" | "yes" | "Y" | "Yes" => {
                        match function::unpack_package(&pkg) {
                            Ok(dir) => {
                                let result = function::check_staging(&dir)
                                    .and_then(|_| function::install_staging(&dir))
                                    .and_then(|_| function::clean_temp(&pkg));

                                if let Err(err) = result {
                                    eprintln!("{}", err);
                                    std::process::exit(1);
                                }
                            }
                            Err(err) => {
                                eprintln!("{}", err);
                                std::process::exit(1);
                            }
                        }
                    },
                    _ => println!("{}", "okay.".yellow()),
                }
            },
            _ => println!("{}", "package not found".red().bold())
        }
    }
    if args.sync {
        match function::sync_main() {
            Ok(()) => {
                println!("{}", "main repository synced".green().bold());
            }
            Err(err) => {
                eprintln!("{}", err.red().bold());
                std::process::exit(1);
            }
        }
    }
}

