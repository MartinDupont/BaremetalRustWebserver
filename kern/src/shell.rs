use stack_vec::StackVec;

use crate::console::{kprint, kprintln, CONSOLE};

use shim::io::Write;
use shim::io::Read;

use core::str;
use core::fmt;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs,
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>,
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// returns if the `exit` command is called.
pub fn shell(prefix: &str) -> ! {
    kprintln!("======================================================================");
    kprintln!("                           Welcome to my OS                           ");
    kprintln!("======================================================================");
    let mut arg_buf = [0u8; 1024];
    loop {
        kprint!("{}", prefix);
        let mut console = &mut *CONSOLE.lock();
        console.read(&mut arg_buf).unwrap();

        let input_str = str::from_utf8(&arg_buf).unwrap();

        let mut command_buf = [""; 10];
        let command = Command::parse(&input_str, &mut command_buf);
        let thing = match command {
            Ok(x) => x,
            Err(_) => continue
        };

        let arg1 = thing.args[1];

        match arg1 {
            "echo" => {
                let arg2 = thing.args[2];
                console.write(arg2.as_bytes());
            }
            _ => {
                console.write("I only accept echo commands".as_bytes());
            }
        };

    }
}
