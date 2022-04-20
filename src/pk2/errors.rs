use std::io;

#[derive(std::fmt::Debug)]
pub enum Error {
    InvalidHeader(&'static str),
    IO(io::Error),
    InvalidBlock(&'static str)
}