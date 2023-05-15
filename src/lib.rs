use ascii::{AsciiString, ToAsciiChar};
use std::fs;
use std::path::Path;

extern crate ascii;
extern crate tiny_http;
use std::ffi::{c_char, CStr, c_uint};

use std::sync::Mutex;
use lazy_static::lazy_static;


lazy_static! {
    static ref FREEPORT: Mutex<u32> = Mutex::new(Some(pick_unused_port()).unwrap().unwrap().into());
}

fn test() {
    let result = FREEPORT.lock().unwrap();
    println!("{}", *result);
}


#[no_mangle]
pub extern fn start_server(to: *const c_char, freeport: u32) {
    let c_str = unsafe { CStr::from_ptr(to) };
    println!("GOT file_root {}", c_str.to_str().unwrap());
    let file_root = c_str.to_str().unwrap().to_string();
    run_server(file_root, freeport as u32);
}

#[no_mangle]
pub extern "C"  fn extern_get_port() -> c_uint {
    return get_port() as c_uint;
}

pub fn get_port() -> u32 {
    let result_guard = FREEPORT.lock().unwrap();
    return *result_guard
}


fn get_content_type(path: &Path) -> &'static str {
    let extension = match path.extension() {
        None => return "text/plain",
        Some(e) => e,
    };

    match extension.to_str().unwrap() {
        "gif" => "image/gif",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "png" => "image/png",
        "pdf" => "application/pdf",
        "htm" => "text/html; charset=utf8",
        "html" => "text/html; charset=utf8",
        "txt" => "text/plain; charset=utf8",
        _ => "text/plain; charset=utf8",
    }
}

fn run_server(file_root: String, freeport: u32) {
    let server_base_url = format!("127.0.0.1:{}", freeport);
    let server = tiny_http::Server::http(server_base_url).unwrap();
    let myport = server.server_addr().to_ip().unwrap().port();
    println!("Now listening on port {}", myport);
    println!("Serving file_root {}", file_root);

    loop {
        let rq = match server.recv() {
            Ok(rq) => rq,
            Err(_) => break,
        };

        println!("{:?}", rq);

        let url = rq.url().to_string();
        let base_path = Path::new(file_root.as_str());
        let path = base_path.join(".".to_owned() +  &url);
        let file = fs::File::open(&path);

        if file.is_ok() {
            let response = tiny_http::Response::from_file(file.unwrap());

            let response = response.with_header(tiny_http::Header {
                field: "Content-Type".parse().unwrap(),
                value: AsciiString::from_ascii(get_content_type(&path)).unwrap(),
            });

            let _ = rq.respond(response);
        } else {
            let rep = tiny_http::Response::new_empty(tiny_http::StatusCode(404));
            let _ = rq.respond(rep);
        }
    }
}

use rand::prelude::*;
use std::net::{
    Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, TcpListener, ToSocketAddrs, UdpSocket,
};

pub type Port = u16;

// Try to bind to a socket using UDP
fn test_bind_udp<A: ToSocketAddrs>(addr: A) -> Option<Port> {
    Some(UdpSocket::bind(addr).ok()?.local_addr().ok()?.port())
}

// Try to bind to a socket using TCP
fn test_bind_tcp<A: ToSocketAddrs>(addr: A) -> Option<Port> {
    Some(TcpListener::bind(addr).ok()?.local_addr().ok()?.port())
}

/// Check if a port is free on UDP
pub fn is_free_udp(port: Port) -> bool {
    let ipv4 = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
    let ipv6 = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0);

    test_bind_udp(ipv6).is_some() && test_bind_udp(ipv4).is_some()
}

/// Check if a port is free on TCP
pub fn is_free_tcp(port: Port) -> bool {
    let ipv4 = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
    let ipv6 = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0);

    test_bind_tcp(ipv6).is_some() && test_bind_tcp(ipv4).is_some()
}

/// Check if a port is free on both TCP and UDP
pub fn is_free(port: Port) -> bool {
    is_free_tcp(port) && is_free_udp(port)
}

/// Asks the OS for a free port
fn ask_free_tcp_port() -> Option<Port> {
    let ipv4 = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
    let ipv6 = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0);

    test_bind_tcp(ipv6).or_else(|| test_bind_tcp(ipv4))
}

/// Picks an available port that is available on both TCP and UDP
/// ```rust
/// use portpicker::pick_unused_port;
/// let port: u16 = pick_unused_port().expect("No ports free");
/// ```
pub fn pick_unused_port() -> Option<Port> {
    let mut rng = rand::thread_rng();

    // Try random port first
    for _ in 0..10 {
        let port = rng.gen_range(15000..25000);
        if is_free(port) {
            return Some(port);
        }
    }

    // Ask the OS for a port
    for _ in 0..10 {
        if let Some(port) = ask_free_tcp_port() {
            // Test that the udp port is free as well
            if is_free_udp(port) {
                return Some(port);
            }
        }
    }

    // Give up
    None
}




#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pick_works() {
        assert!(pick_unused_port().is_some());
    }

    #[test]
    fn it_works() {
        run_server("".to_string(), get_port());
    }
}
