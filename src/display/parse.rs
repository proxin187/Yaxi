use crate::display::error::*;

use std::iter::Peekable;
use std::str::Chars;
use std::env;

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

#[derive(PartialEq)]
pub enum State {
    Host,
    Protocol,
    Display,
    Screen,
    Finished,
}

pub struct Iter<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Iter<'a> {
    pub fn new(chars: Peekable<Chars<'a>>) -> Iter {
        Iter {
            chars,
        }
    }

    pub fn take_while<F: Fn(&char) -> bool>(&mut self, f: F) -> Result<Vec<char>, Error> {
        let mut buf: Vec<char> = Vec::new();

        while self.chars.peek().map(|c| f(c)).unwrap_or(false) {
            buf.push(self.chars.next().ok_or(Error::InvalidDisplay)?);
        }

        Ok(buf)
    }

    pub fn next_option(&mut self) -> Option<char> { self.chars.next() }

    pub fn next(&mut self) -> Result<char, Error> {
        self.chars.next().ok_or(Error::InvalidDisplay)
    }

    pub fn expect(&mut self, expect: char) -> Result<(), Error> {
        self.next().and_then(|c| (c != expect).then(|| Err(Error::InvalidDisplay)).unwrap_or(Ok(())))
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
                    self.display.host = self.iter.take_while(|c| *c != ':' && *c != '/')?.iter().collect();

                    match self.iter.next()? {
                        ':' => self.state = State::Display,
                        '/' => self.state = State::Protocol,
                        _ => unreachable!(),
                    }
                },
                State::Protocol => {
                    self.display.protocol = Protocol::from(self.iter.take_while(|c| *c != ':')?.iter().collect())?;

                    self.state = State::Display;
                },
                State::Display => {
                    self.display.display = self.iter.take_while(|c| *c != '.')?.iter().collect::<String>().parse::<u16>().map_err(|_| Error::InvalidDisplay)?;

                    match self.iter.next_option() {
                        Some('.') => self.state = State::Screen,
                        _ => self.state = State::Finished,
                    }
                },
                State::Screen => {
                    self.display.screen = self.iter.take_while(|c| *c != '.')?.iter().collect::<String>().parse::<u16>().map_err(|_| Error::InvalidDisplay)?;
                },
                _ => {},
            }
        }

        Ok(self.display.clone())
    }
}

// TODO: FINSIH THIS
pub fn parse<'a>(display: &'a str) {
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let display = Parser::parse(Some(String::from(":69"))).unwrap();

        assert_eq!(display, ParseDisplay::new(None, Protocol::UnixSocket, 69, 0));
    }

    #[test]
    fn test_unix() {
        let display = ParseDisplay::parse(Some(String::from("unix:/some/file/path"))).unwrap();

        assert_eq!(display, ParseDisplay::new(Some(String::from("/some/file/path")), Protocol::UnixSocket, 0, 0));
    }

    #[test]
    fn test_tcp() {
        let display = ParseDisplay::parse(Some(String::from("13.37.13.37/tcp:69.420"))).unwrap();

        assert_eq!(display, ParseDisplay::new(Some(String::from("/some/file/path")), Protocol::UnixSocket, 0, 0));
    }
}
*/

