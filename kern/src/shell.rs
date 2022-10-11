use shim::io;
use shim::path::{Path, PathBuf, Component};

use pi::atags::Atags;

use fat32::traits::FileSystem;
use fat32::traits::{Dir, Entry, File};

use crate::console::{kprint, kprintln, CONSOLE};
use crate::{ALLOCATOR, FILESYSTEM};

use shim::io::Write;
use shim::io::Read;

use alloc::vec::Vec;

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
    args: Vec<&'a str>,
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
        let mut args = Vec::new();
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg);
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


struct Shell {
    cwd: PathBuf,
}

impl Shell {
    pub fn new() -> Shell {
        Shell { cwd: PathBuf::from("/") }
    }

    fn _shell(&mut self, prefix: &str) -> ! {
        const CMD_LEN: usize = 512;
        const ARG_LEN: usize = 64;
        kprintln!();
        kprintln!("======================================================================");
        kprintln!("                           Welcome to my OS                           ");
        kprintln!("======================================================================");
        kprintln!();
        loop {
            let mut cmd_buf = [0u8; CMD_LEN];
            let mut arg_buf = [""; ARG_LEN];

            kprint!("{} {}", self.cwd.to_str().unwrap(), prefix);

            let mut i = 0;
            'cmd: loop {
                if i == CMD_LEN {
                    kprintln!();
                    kprintln!("command length exceeds {}", CMD_LEN);
                    break 'cmd;
                }

                let byte = CONSOLE.lock().read_byte();
                if byte == b'\n' || byte == b'\r' {
                    kprint!("\n");
                    let cmd_result = str::from_utf8(&cmd_buf[0..i]);
                    if let Ok(cmd) = cmd_result {
                        match Command::parse(cmd, &mut arg_buf) { // enter
                            Err(Error::Empty) => {}
                            Err(Error::TooManyArgs) => {
                                kprintln!("error: too many arguments");
                            }
                            Ok(cmd) => {
                                self.process_command(cmd);
                            }
                        }
                        break 'cmd;
                    } else {
                        kprintln!("Could not parse input bytes into string");
                        kprint!("\u{7}");
                        kprintln!("");
                        cmd_buf = [0u8; CMD_LEN];
                        arg_buf = [""; ARG_LEN];
                    }
                } else if byte == 8 || byte == 127 { // backspace
                    if i > 0 {
                        kprint!("\u{8} \u{8}");
                        i -= 1
                    }
                } else {
                    cmd_buf[i] = byte;
                    CONSOLE.lock().write_byte(byte);
                    i += 1;
                }
            }
        }
    }


    fn process_command(&mut self, mut cmd: Command) -> Option<()> {
        let arg1 = cmd.args.remove(0);
        match arg1 {
            "echo" => {
                for arg in cmd.args {
                    kprint!("{} ", arg);
                }
                kprint!("\n");
            }
            "pwd" => {
                kprintln!("{}", self.cwd.to_str().unwrap());
            }
            "ls" => { self.ls(cmd.args) }
            "cd" => { self.cd(cmd.args) }
            "cat" => { self.cat(cmd.args) }
            "" => {
                kprintln!();
            }
            _ => {
                kprintln!("Unknown command: {}", arg1);
            }
        };
        return Some(());
    }


    fn cd(&mut self, mut args: Vec<&str>) {
        if args.len() != 1 {
            kprintln!("cd takes only 1 argument, but received {}", args.len());
            return;
        }

        let arg = args.remove(0);
        match FILESYSTEM.open(self.get_entry(arg)) {
            Err(_) => kprintln!("Error opening {}", arg),
            Ok(entry) => {
                if entry.is_dir() {
                    self.cwd = self.get_entry(arg);
                } else {
                    kprintln!("{} is not a directory", arg);
                }
            }
        }
    }


    fn cat(&self, mut args: Vec<&str>) {
        if args.len() == 0 {
            kprintln!("expected at least one argument");
        }

        for arg in args {
            match FILESYSTEM.open(self.get_entry(arg)) {
                Ok(entry) => match entry.into_file() {
                    Some(mut file) => {
                        let mut file_contents = Vec::new();
                        for _ in 0..file.size() {
                            file_contents.push(0);
                        }
                        match file.read(file_contents.as_mut_slice()) {
                            Ok(bytes_read) => {
                                if bytes_read < file.size() as usize {
                                    kprintln!("Could only read {} of {} bytes in {}",
                                            bytes_read, file.size(), arg);
                                } else {
                                    match core::str::from_utf8(file_contents.as_slice()) {
                                        Ok(contents) => kprintln!("{}", contents),
                                        Err(_) => kprintln!("{} contains non-UTF8 characters", arg),
                                    }
                                }
                            }
                            Err(_) => kprintln!("Error reading the contents of {}", arg),
                        }
                    },
                    None => kprintln!("{} is a directory", arg),
                }
                Err(_) => kprintln!("Error opening {}", arg),
            }
        }
    }

    fn ls(&self, mut args: Vec<&str>) {
        let mut display_hidden = false;
        if args.len() > 0 {
            let arg = args.remove(0);
            if arg == "-a" {
                display_hidden = true;
            } else {
                args.insert(0, arg);
            }
        }

        let ls_dir = |path: &PathBuf| {
            match FILESYSTEM.open(path) {
                Ok(entry) => match entry.as_dir() {
                    Some(dir) => {
                        match dir.entries() {
                            Ok(entries) => {
                                for entry in entries {
                                    if display_hidden || !entry.metadata().attributes.hidden() {
                                        if entry.metadata().attributes.directory() || entry.metadata().attributes.archive(){
                                            kprintln!("{}", entry.name());
                                        }
                                    }
                                }
                            }
                            Err(_) => kprintln!("Cannot open directory {}", path.to_str().unwrap()),
                        }
                    }
                    None => kprintln!("{}", entry.name()),
                }
                Err(_) => kprintln!("Cannot open directory {}", path.to_str().unwrap()),
            };
        };

        if args.len() == 0 {
            // ls in cwd
            ls_dir(&self.cwd);
        } else {
            // ls each argument
            for arg in args {
                ls_dir(&self.get_entry(arg));
            }
        }
    }

    // Gets the entries identified by the given path.
    fn get_entry(&self, path: &str) -> PathBuf {
        let mut curr = self.cwd.clone();
        let path = PathBuf::from(path);

        for component in path.components() {
            match component {
                Component::RootDir => curr = PathBuf::from("/"),
                Component::ParentDir => { curr.pop(); }
                Component::Normal(entry) => curr.push(entry),
                _ => (), // Nothing to do for `Prefix` or `CurDir`
            }
        }
        curr
    }
}


/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns.
pub fn shell(prefix: &str) -> ! {
    let mut the_shell = Shell::new();
    the_shell._shell(prefix)
}
