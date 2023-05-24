use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

use ssh2::Session;

fn main() {
    // Connect to the local SSH.
    let mut session = Session::new().unwrap();

    session.set_tcp_stream(TcpStream::connect(format!("{}:{}", "localhost", 22)).unwrap());
    session.handshake().unwrap();
    session.userauth_password(std::env::var("SSH_USERNAME").unwrap().as_str(), std::env::var("SSH_PASSWORD").unwrap().as_str()).unwrap();

    // Send a file.
    let mut remote_file = session.scp_send(Path::new("remote.txt"),
                                        0o644, 10, None).unwrap();
    remote_file.write_all(b"1234567890").unwrap();
    remote_file.send_eof().unwrap();
    remote_file.wait_eof().unwrap();
    remote_file.close().unwrap();
    remote_file.wait_close().unwrap();

    // Send a folder.
    let mut remote_folder = session.scp_send(Path::new("remote_folder"),
                                        0o755, 0, None).unwrap();
    remote_folder.send_eof().unwrap();
    remote_folder.wait_eof().unwrap();
    remote_folder.close().unwrap();
    remote_folder.wait_close().unwrap();
}
