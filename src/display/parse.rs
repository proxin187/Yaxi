use crate::display::error::*;

use std::env;
use std::iter::Peekable;
use std::str::Chars;

// TODO: currently cant find the specification for DISPLAY so this might not be 100% accurate

/// represents which protocol the x11 connection should use

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Protocol {
    TcpSocket,

    #[default]
    UnixSocket,
}

impl Protocol {
    pub fn from(value: String) -> Result<Protocol, Error> {
        match value.as_str().to_lowercase().trim_end_matches(":") {
            "unix" => Ok(Protocol::UnixSocket),
            "tcp" => Ok(Protocol::TcpSocket),
            _ => Err(Error::InvalidProtocol { protocol: value }),
        }
    }
}

/// representing the $DISPLAY environment variable
/// syntax: <host>/<protocol>:<display>.<screen>

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DisplayInfo {
    pub host: String,
    pub protocol: Protocol,
    pub display: u16,
    pub screen: u16,
}

impl DisplayInfo {
    pub fn new(host: String, protocol: Protocol, display: u16, screen: u16) -> DisplayInfo {
        DisplayInfo {
            host,
            protocol,
            display,
            screen,
        }
    }
}

#[derive(PartialEq)]
pub enum State {
    Host,
    Protocol,
    Display,
    Screen,
    Unix,
    Finished,
}

pub struct Iter<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Iter<'a> {
    pub fn new(chars: Peekable<Chars<'a>>) -> Iter {
        Iter { chars }
    }

    pub fn take_while<F: Fn(&char) -> bool>(&mut self, f: F) -> Result<Vec<char>, Error> {
        let mut buf: Vec<char> = Vec::new();

        while self.chars.peek().map(|c| f(c)).unwrap_or(false) {
            buf.push(self.chars.next().ok_or(Error::InvalidDisplay)?);
        }

        Ok(buf)
    }

    pub fn next_option(&mut self) -> Option<char> {
        self.chars.next()
    }

    pub fn next(&mut self) -> Result<char, Error> {
        self.chars.next().ok_or(Error::InvalidDisplay)
    }

    pub fn expect(&mut self, expect: char) -> Result<(), Error> {
        self.next().and_then(|c| {
            (c != expect)
                .then(|| Err(Error::InvalidDisplay))
                .unwrap_or(Ok(()))
        })
    }
}

/// this parses the $DISPLAY environment variable using a state machine
pub struct Parser<'a> {
    iter: Iter<'a>,
    display: DisplayInfo,
    state: State,
}

impl<'a> Parser<'a> {
    pub fn new(display: &'a str) -> Parser<'a> {
        Parser {
            iter: Iter::new(display.chars().peekable()),
            display: DisplayInfo::default(),
            state: State::Host,
        }
    }

    pub fn parse(&mut self) -> Result<DisplayInfo, Error> {
        while self.state != State::Finished {
            match self.state {
                State::Host => {
                    self.display.host = self
                        .iter
                        .take_while(|c| *c != ':' && *c != '/')?
                        .iter()
                        .collect();

                    match (self.display.host.as_str(), self.iter.next()?) {
                        ("unix", _) => self.state = State::Unix,
                        (_, ':') => self.state = State::Display,
                        (_, '/') => self.state = State::Protocol,
                        _ => unreachable!(),
                    }
                }
                State::Protocol => {
                    self.display.protocol =
                        Protocol::from(self.iter.take_while(|c| *c != ':')?.iter().collect())?;

                    self.iter.next()?;

                    self.state = State::Display;
                }
                State::Display => {
                    self.display.display = self
                        .iter
                        .take_while(|c| *c != '.')?
                        .iter()
                        .collect::<String>()
                        .parse::<u16>()
                        .map_err(|_| Error::InvalidDisplay)?;

                    match self.iter.next_option() {
                        Some('.') => self.state = State::Screen,
                        _ => self.state = State::Finished,
                    }
                }
                State::Screen => {
                    self.display.screen = self
                        .iter
                        .take_while(|c| *c != '.')?
                        .iter()
                        .collect::<String>()
                        .parse::<u16>()
                        .map_err(|_| Error::InvalidDisplay)?;

                    self.state = State::Finished;
                }
                State::Unix => {
                    self.display.host = self.iter.take_while(|_| true)?.iter().collect();

                    self.state = State::Finished;
                }
                _ => {}
            }
        }

        Ok(self.display.clone())
    }
}

/// parse the DISPLAY env or provided string
pub fn parse<'a>(display: Option<&'a str>) -> Result<DisplayInfo, Error> {
    let env = env::var("DISPLAY").map_err(|_| Error::InvalidDisplay)?;

    Parser::new(display.unwrap_or(&env)).parse()
}

#[cfg(test)]
mod tests {
    use crate::display::parse::{self, *};

    #[test]
    fn test_simple() -> Result<(), Error> {
        let display = parse::parse(Some(":69"))?;

        assert_eq!(
            display,
            DisplayInfo::new(String::new(), Protocol::UnixSocket, 69, 0)
        );

        Ok(())
    }

    #[test]
    fn test_unix() -> Result<(), Error> {
        let display = parse::parse(Some("unix:/some/file/path"))?;

        assert_eq!(
            display,
            DisplayInfo::new(String::from("/some/file/path"), Protocol::UnixSocket, 0, 0)
        );

        Ok(())
    }

    #[test]
    fn test_tcp() -> Result<(), Error> {
        let display = parse::parse(Some("13.37.13.37/tcp:69.420"))?;

        assert_eq!(
            display,
            DisplayInfo::new(String::from("13.37.13.37"), Protocol::TcpSocket, 69, 420)
        );

        Ok(())
    }
}
