use clap::{App, Arg};
use ssh2::Session;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;

fn main() {
    // Parse command-line arguments
    let matches = App::new("Docker Installer")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Downloads and installs a Docker image on a remote server")
        .arg(
            Arg::with_name("image")
                .help("The Docker image name")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("ip")
                .help("The IP address of the destination server")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("key")
                .help("The path to the SSH private key")
                .required(true)
                .index(3),
        )
        .arg(
            Arg::with_name("user")
                .help("The SSH username")
                .required(true)
                .index(4),
        )
        .get_matches();

    let image = matches.value_of("image").unwrap();
    let ip = matches.value_of("ip").unwrap();
    let key_path = matches.value_of("key").unwrap();
    let user = matches.value_of("user").unwrap();

    // Connect to the remote server via SSH
    let tcp = TcpStream::connect(format!("{}:22", ip)).expect("Failed to connect to the server");
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    // Authenticate with the server using a private key
    sess.userauth_pubkey_file(user, None, Path::new(key_path), None)
        .expect("Failed to authenticate with private key");

    // Check if authentication was successful
    if !sess.authenticated() {
        eprintln!("Authentication failed");
        return;
    }

    // Pull the Docker image on the remote server
    let mut channel = sess.channel_session().unwrap();
    channel
        .exec(&format!("docker pull {}", image))
        .expect("Failed to execute Docker pull command on the remote server");

    let mut stdout = String::new();
    let mut stderr = String::new();
    channel.read_to_string(&mut stdout).unwrap();
    channel.stderr().read_to_string(&mut stderr).unwrap();

    println!("Standard Output:\n{}", stdout);
    if !stderr.is_empty() {
        eprintln!("Standard Error:\n{}", stderr);
    }

    channel.wait_close().unwrap();
    println!("Docker pull command executed with exit status: {}", channel.exit_status().unwrap());

    // Run the Docker container on the remote server
    let mut channel = sess.channel_session().unwrap();
    channel
        .exec(&format!("docker run -d {}", image))
        .expect("Failed to execute Docker run command on the remote server");

    let mut stdout = String::new();
    let mut stderr = String::new();
    channel.read_to_string(&mut stdout).unwrap();
    channel.stderr().read_to_string(&mut stderr).unwrap();

    println!("Standard Output:\n{}", stdout);
    if !stderr.is_empty() {
        eprintln!("Standard Error:\n{}", stderr);
    }

    channel.wait_close().unwrap();
    println!("Docker run command executed with exit status: {}", channel.exit_status().unwrap());
}