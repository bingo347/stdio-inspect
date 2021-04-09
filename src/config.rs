use env::ArgsOs;
use std::env;
use std::ffi::OsString;
use std::net::SocketAddr;
use std::process;

use crate::command::Command;

pub fn collect_config() -> AppConfig {
    AppConfig::collect()
}

#[derive(Debug)]
pub enum AppConfig {
    Run {
        command: Command,
        udp: Option<SocketAddr>,
        gui: bool,
    },
    View {
        udp: SocketAddr,
        gui: bool,
    },
}

macro_rules! print_usage {
    () => {
        print_usage!(0)
    };
    ($code:expr) => {{
        eprint!(
            "Standard IO inspecting tool

USAGE:
    stdio-inspect [OPTIONS] <executable> [ARGS]
    stdio-inspect [OPTIONS] -

OPTIONS:
    --gui                     - Show in GUI
    --host <HOST>, -h <HOST>  - UDP hostname
    --port <PORT>, -p <PORT>  - UDP port
    --help, -?                - Print this help

EXAMPLES:
    stdio-inspect --gui <executable>
        - inspect executable and show it stdio in GUI
    stdio-inspect --port 9000 <executable>
        - inspect executable and send it stdio to UPD port 9000
    stdio-inspect --port 9000 -
        - listen UPD port 9000 and print it to stdout
    stdio-inspect --port 9000 --gui -
        - listen UPD port 9000 and show it in GUI
"
        );
        process::exit($code)
    }};
}
macro_rules! error {
    ($msg:expr) => {{
        eprint!("Error: ");
        eprintln!($msg);
        eprintln!();
        print_usage!(-1)
    }};
    ($fmt:expr, $($arg:tt)+) => {{
        eprint!("Error: ");
        eprintln!($fmt, $($arg)+);
        eprintln!();
        print_usage!(-1)
    }};
}

impl AppConfig {
    fn collect() -> Self {
        let mut view_only = false;
        let mut gui = false;
        let mut host = None;
        let mut port = None;
        let command = read_args(|arg, args| match arg {
            "-" => view_only = true,
            "--gui" => gui = true,
            "--host" | "-h" => host = args.next(),
            "--port" | "-p" => port = args.next(),
            "--help" | "-?" => print_usage!(),
            _ => error!("Unknown argument {}", arg),
        });
        let udp = make_udp(host, port, view_only);
        match (view_only, command, udp) {
            (true, _, None) => error!("View mode required udp port"),
            (false, None, _) => error!("<executable> required"),
            (false, Some(command), udp) => AppConfig::Run { command, udp, gui },
            (true, _, Some(udp)) => AppConfig::View { udp, gui },
        }
    }
}

fn read_args<Matcher>(mut matcher: Matcher) -> Option<Command>
where
    Matcher: FnMut(&str, &mut ArgsOs),
{
    macro_rules! ret {
        () => {
            return None;
        };
        ($input:expr) => {
            return Command::new($input);
        };
    }
    let mut args = env::args_os();
    args.next().expect("First argument always some executable");
    loop {
        match args.next() {
            Some(arg) => match arg.into_string() {
                Ok(arg) => {
                    if !arg.starts_with('-') {
                        ret!((arg, args))
                    }
                    match arg.as_ref() {
                        "--" => ret!(args),
                        other => matcher(other, &mut args),
                    }
                }
                Err(command) => ret!((command, args)),
            },
            _ => ret!(),
        }
    }
}

fn make_udp(host: Option<OsString>, port: Option<OsString>, view_only: bool) -> Option<SocketAddr> {
    use std::net::{IpAddr, Ipv6Addr};
    fn err(arg_name: &str) -> ! {
        error!("Invalid argument {}", arg_name);
    }
    match port {
        Some(port) => {
            let port = port.into_string().unwrap_or_else(|_| err("port"));
            let port: u16 = port.parse().unwrap_or_else(|_| err("port"));
            Some(SocketAddr::new(
                match host {
                    Some(host) => {
                        let host = host.into_string().unwrap_or_else(|_| err("host"));
                        if host == "localhost" {
                            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
                        } else {
                            host.parse().unwrap_or_else(|_| err("host"))
                        }
                    }
                    _ => IpAddr::V6(Ipv6Addr::new(
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        if view_only { 0 } else { 1 },
                    )),
                },
                port,
            ))
        }
        _ => match host {
            None => None,
            _ => error!("Host argument without port is not supported"),
        },
    }
}
