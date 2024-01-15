use std::time::Duration;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    link: String,
    #[arg(short, long, default_value_t=9600 ,value_parser=clap::value_parser!(u32).range(1024..115200),)]
    baud_rate: u32,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let port = serialport::new(args.link, args.baud_rate)
        .timeout(Duration::from_millis(10))
        .open();

    match port {
        Ok(mut port) => {

            let mut epson = epsonemu::epsonlib::Epsonlib::new(&mut port);

            epson.run_until();
        }
        Err(e) => {
            eprintln!("{:?}", e);
        }
    }

    Ok(())
}