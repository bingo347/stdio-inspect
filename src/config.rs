use std::env;
use std::net::SocketAddr;
use std::ffi::OsString;

#[derive(Debug)]
pub struct AppConfig {
    pub args: env::ArgsOs,
    pub command: Option<OsString>,
    pub view_only: bool,
    pub gui: bool,
    pub udp: Option<SocketAddr>
}

impl AppConfig {
    fn new() -> Self {
        Self {
            args: env::args_os(),
            command: None,
            view_only: false,
            gui: false,
            udp: None
        }
    }

    fn read_args(mut self) -> Self {
        self.args.next().expect("First argument always some executable");
        loop {
            match self.args.next() {
                Some(arg) => match arg.into_string() {
                    Ok(arg) => {
                        if !arg.starts_with('-') {
                            self.command = Some(arg.into());
                            break;
                        }
                        match arg.as_ref() {
                            "-" => {
                                self.view_only = true;
                                break;
                            },
                            "--" => {
                                self.command = self.args.next();
                                break;
                            },
                            "--help" | "-?" => {
                                print_usage();
                                std::process::exit(0);
                            },
                            "--gui" => {
                                self.gui = true;
                            },
                            "--port" | "-p" => {
                                if !update_address(&mut self.udp, None, self.args.next()) {
                                    print_usage();
                                    panic!("Port argument without value");
                                }
                            },
                            "--host" | "-h" => {
                                if !update_address(&mut self.udp, self.args.next(), None) {
                                    print_usage();
                                    panic!("Host argument without value");
                                }
                            },
                            _ => panic!("Unknown argument {}", arg)
                        }
                    },
                    Err(command) => {
                        self.command = Some(command);
                        break;
                    }
                },
                None => break
            }
        }
        self.check_continue();
        self
    }

    fn check_continue(&self) {
        if let Some(ref udp) = self.udp {
            if udp.port() == 0 {
                panic!("Host argument without port is not supported");
            }
        }
        if !self.view_only && self.command == None {
            print_usage();
            std::process::exit(-1);
        }
    }
}

pub(crate) fn collect_config() -> AppConfig {
    AppConfig::new().read_args()
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

fn print_usage() {
    todo!("print_usage");
}