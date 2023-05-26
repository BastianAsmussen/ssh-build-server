use std::path::Path;

use ssh2::Session;

use crate::util::settings::Settings;
use crate::util::ssh::Sbs;

mod util;

fn main() {
    // Get the arguments passed to the program.
    let args: Vec<String> = std::env::args().collect();

    // The first user-supplied argument is the path to the config file.
    let config_path = match args.get(1) {
        Some(path) => path,
        None => {
            eprintln!("No config file path was supplied, using default...");

            ""
        }
    };

    // Load the config.
    println!("Loading config...");
    let settings = Settings::new(config_path);
    let settings = match settings {
        Ok(settings) => settings,
        Err(err) => {
            eprintln!("Failed to load config: {}", err);

            return;
        }
    };

    // Connect to the local SSH.
    println!("Connecting to SSH...");
    let mut sbs = Sbs::new(Session::new().unwrap());

    let connect_operation = sbs.connect(
        &settings.ssh.host,
        &settings.ssh.port,
        &settings.ssh.username,
        &settings.ssh.password,
    );
    match connect_operation {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Failed to connect to SSH: {}", err);

            return;
        }
    }

    // Clone the directory to the local SSH.
    println!("Copying project to remote... ({} -> {})", settings.compilation.local_project_root, settings.compilation.remote_project_root);
    let send_operation = sbs.send_directory(
        Path::new(&settings.compilation.local_project_root),
        Path::new(&settings.compilation.remote_project_root),
    );
    match send_operation {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Failed to copy project: {}", err);

            return;
        }
    }

    // Make the SSH server execute the commands.
    println!("Compiling code...");
    let compile_operation = sbs.execute_commands(&settings.commands.to_vec(), false);
    match compile_operation {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Failed to compile code: {}", err);

            return;
        }
    }

    // Download the output folder from the SSH server.
    println!("Downloading output folder...");
    let download_operation = sbs.receive_directory(
        Path::new(&settings.compilation.get_local_output_directory()),
        Path::new(&settings.compilation.get_remote_output_directory()),
    );
    match download_operation {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Failed to download output folder: {}", err);

            return;
        }
    }

    // Execute post-compilation commands.
    println!("Executing post-compilation commands...");
    let post_compile_operation = sbs.execute_commands(&settings.commands.to_vec(), true);
    match post_compile_operation {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Failed to execute post-compilation commands: {}", err);

            return;
        }
    }

    // Disconnect from the SSH server.
    println!("Disconnecting from SSH...");
    let disconnect_operation = sbs.disconnect(None, "", None);
    match disconnect_operation {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Failed to disconnect from SSH: {}", err);
        }
    }
}
