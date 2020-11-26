use std::env;
use std::net::SocketAddr;
use std::ffi::OsString;
use std::process;
use crate::command::Command;

pub fn collect_config() -> AppConfig {
    AppConfig::collect()
}

#[derive(Debug)]
pub struct AppConfig {
    pub command: Option<Command>,
    pub view_only: bool,
    pub gui: bool,
    pub udp: Option<SocketAddr>
}

macro_rules! print_usage {
    () => { print_usage!(0) };
    ($code:expr) => {{
        eprint!("Standard IO inspecting tool

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
");
        process::exit($code);
    }};
}
macro_rules! error {
    ($msg:expr) => {{
        eprint!("Error: ");
        eprintln!($msg);
        eprintln!();
        print_usage!(-1);
    }};
    ($fmt:expr, $($arg:tt)+) => {{
        eprint!("Error: ");
        eprintln!($fmt, $($arg)+);
        eprintln!();
        print_usage!(-1);
    }};
}

impl AppConfig {
    fn collect() -> Self {
        let mut view_only = false;
        let mut gui = false;
        let (command, udp) = read_args(|arg| match arg {
            "-" => view_only = true,
            "--gui" => gui = true,
            "--help" | "-?" => print_usage!(),
            _ => error!("Unknown argument {}", arg)
        });
        Self { command, view_only, gui, udp }.check_continue()
    }

    fn check_continue(self) -> Self {
        if let Some(ref udp) = self.udp {
            if udp.port() == 0 {
                error!("Host argument without port is not supported");
            }
        }
        if !self.view_only && self.command.is_none() {
            print_usage!(-1);
        }
        self
    }
}

fn read_args(mut matcher: impl FnMut(&str)) -> (Option<Command>, Option<SocketAddr>) {
    let mut udp = None;
    macro_rules! ret {
        () => { return (None, udp); };
        ($input:expr) => { return (Command::new($input), udp); };
    }
    let mut args = env::args_os();
    args.next().expect("First argument always some executable");
    loop {
        match args.next() {
            Some(arg) => match arg.into_string() {
                Ok(arg) => {
                    if !arg.starts_with('-') { ret!((arg, args)) }
                    match arg.as_ref() {
                        "--" => ret!(args),
                        "--port" | "-p" => {
                            if !update_address(&mut udp, None, args.next()) {
                                error!("Port argument without value");
                            }
                        },
                        "--host" | "-h" => {
                            if !update_address(&mut udp, args.next(), None) {
                                error!("Host argument without value");
                            }
                        },
                        other => matcher(other)
                    }
                },
                Err(command) => ret!((command, args))
            },
            _ => ret!()
        }
    }
}

fn update_address(addr: &mut Option<SocketAddr>, host: Option<OsString>, port: Option<OsString>) -> bool {
    match *addr {
        Some(ref mut addr) => {
            if let Some(host) = host {
                use std::net::{IpAddr, Ipv6Addr};
                const ERR: &str = "Invalid argument host";
                let host = host.into_string().expect(ERR);
                let host: IpAddr = if host == "localhost" {
                    IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
                } else {
                    host.parse().expect(ERR)
                };
                addr.set_ip(host);
                true
            } else if let Some(port) = port {
                const ERR: &str = "Invalid argument port";
                let port = port.into_string().expect(ERR);
                let port: u16 = port.parse().expect(ERR);
                addr.set_port(port);
                true
            } else {
                false
            }
        },
        None => {
            *addr = Some(SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], 0)));
            update_address(addr, host, port)
        }
    }
}
