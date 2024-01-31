use std::time::Duration;
use clap::{
    Parser,
    Subcommand
};
use serialport::{
    SerialPortType::UsbPort,
    available_ports,
};


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Retrieves all the available ports
    Ports,
    /// Open a given port for emulation
    Open {
        #[arg(short, long)]
        port: String,
        #[arg(short, long, default_value_t=9600 ,value_parser=clap::value_parser!(u32).range(1024..115200),)]
        baud_rate: u32,
        #[arg(short, long, default_value_t=20)]
        warming: u32,
        #[arg(short, long, default_value_t=5)]
        cooling: u32,
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    if let Some(command) = args.command {
        match command {
            Commands::Ports => {
                let ports = available_ports().map_err(|e |{
                    eprintln!("{e:?}");
                    std::io::Error::new(std::io::ErrorKind::Other, "Error getting available ports")
                })?.iter()
                    .map(|p| {
                        if let UsbPort(port) = &p.port_type {
                            if let Some(manufacturer) = port.manufacturer.as_ref() {
                                format!("{} - {}", p.port_name, manufacturer)
                            } else {
                                format!("{} - USB", p.port_name)
                            }
                        }
                        else {
                            format!("{} - Unknown", p.port_name)
                        }
                    })
                    .collect::<Vec<String>>();

                println!("Available ports:\n{}", ports.join("\n"));
            },
            Commands::Open { port, baud_rate,warming, cooling } => {
                let port = serialport::new(port, baud_rate)
                    .timeout(Duration::from_secs(60))
                    .open()?;

                escvp21emulator::escvp21::start(port, cooling, warming);
            }
        }
    }

    Ok(())
}