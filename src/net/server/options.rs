// use std::str::FromStr;
// use crate::Engine;
// use crate::net::server::ServerOpt;
//
// pub fn with_host(host: &'static str) -> fn(&mut Engine) {
//     return |engine| engine.bind_host = host
// }
//
// pub fn with_port(port: u16) -> fn(&mut Engine) {
//     return |engine| engine.bind_port = port
// }
//
// pub fn with_port_str(port: &'static str) -> fn(&mut Engine) {
//     return |engine| engine.bind_port = u16::from_str(port).expect("a valid port value")
// }