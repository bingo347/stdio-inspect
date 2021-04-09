use std::{
    convert::{TryFrom, TryInto},
    env::ArgsOs,
    ffi::OsString,
};

#[derive(Debug)]
pub struct Command {
    command: OsString,
    args: ArgsOs,
}

impl Command {
    pub fn new<T: TryInto<Command>>(input: T) -> Option<Self> {
        match input.try_into() {
            Ok(command) => Some(command),
            _ => None,
        }
    }

    pub fn run(&self) {}
}

macro_rules! impl_try_from {
    ($t:ty => |$input:pat| $body:block) => {
        impl TryFrom<$t> for Command {
            type Error = ();
            fn try_from($input: $t) -> Result<Self, ()> {
                $body
            }
        }
    };
}
impl_try_from![ArgsOs => |mut args| {
    match args.next() {
        Some(command) => Ok(Self { command, args }),
        _ => Err(())
    }
}];
impl_try_from![(OsString, ArgsOs) => |(command, args)| {
    Ok(Self { command, args })
}];
impl_try_from![(String, ArgsOs) => |(command, args)| {
    let command = command.into();
    Ok(Self { command, args })
}];
