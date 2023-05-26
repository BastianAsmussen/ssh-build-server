use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

use ssh2::{DisconnectCode, Session, Sftp};

use crate::util::settings::Command;

pub struct Sbs {
    pub session: Session,
}

impl Sbs {
    /// Creates a new SBS instance.
    ///
    /// # Arguments
    ///
    /// * `session` - The SSH session.
    ///
    /// # Examples
    ///
    /// ```
    /// let session = Session::new().unwrap(); // Your SSH session.
    ///
    /// let sbs = Sbs::new(session);
    /// ```
    pub fn new(session: Session) -> Self {
        Self {
            session,
        }
    }

    /// Connects to the SSH server with the given credentials.
    ///
    /// # Arguments
    ///
    /// * `host` - The host.
    /// * `port` - The port.
    /// * `username` - The username.
    /// * `password` - The password.
    ///
    /// # Examples
    ///
    /// ```
    /// let sbs = Sbs::new(session); // Your SBS instance.
    ///
    /// sbs.connect("localhost", &22, "username", "password").unwrap();
    /// ```
    pub fn connect(&mut self, host: &str, port: &u16, username: &str, password: &str) -> Result<(), Error> {
        let address = format!("{}:{}", host, port);

        self.session.set_tcp_stream(TcpStream::connect(address)?);
        self.session.handshake()?;
        self.session.userauth_password(username, password)?;

        Ok(())
    }

    /// Disconnects from the SSH server.
    ///
    /// # Examples
    ///
    /// ```
    /// let sbs = Sbs::new(session); // Your SBS instance.
    ///
    /// sbs.disconnect(None, "", None).unwrap();
    /// ```
    pub fn disconnect(&mut self, reason: Option<DisconnectCode>, description: &str, lang: Option<&str>) -> Result<(), Error> {
        self.session.disconnect(reason, description, lang)?;

        Ok(())
    }

    /// Compiles a list of commands into a single string.
    ///
    /// # Arguments
    ///
    /// * `commands` - The commands.
    ///
    /// # Examples
    ///
    /// ```
    /// let sbs = Sbs::new(session); // Your SBS instance.
    ///
    /// let commands = vec![
    ///    "ls",
    ///    "cd /test",
    ///    "ls",
    /// ];
    ///
    /// let compiled = sbs.compile_commands(&commands);
    /// ```
    fn compile_commands(&self, commands: &Vec<Command>) -> String {
        let mut compiled = String::new();

        for command in commands {
            compiled.push_str(command.command.as_str());
            compiled.push('\n');
        }

        compiled
    }

    /// Sends a list of commands to the SSH server and returns the output.
    ///
    /// # Arguments
    ///
    /// * `commands` - The commands.
    /// * `is_after_compilation` - Whether this function is called before or after program compilation.
    ///
    /// # Examples
    ///
    /// ```
    /// let sbs = Sbs::new(session); // Your SBS instance.
    ///
    /// let commands = vec![
    ///     "ls",
    ///     "cd /test",
    ///     "ls",
    /// ];
    ///
    /// let output = sbs.execute_commands(&commands, false).unwrap();
    /// ```
    pub fn execute_commands(&self, commands: &[Command], is_after_compilation: bool) -> Result<String, Error> {
        // If it's after compilation, we remove the commands that are before compilation.
        let mut commands = commands.to_vec();
        // For each command that does not match is_after_compilation, remove it.
        commands.retain(|command| command.execute_after_compilation == is_after_compilation);

        // Compile the commands into a single string.
        let compiled_commands = self.compile_commands(&commands);

        let mut channel = self.session.channel_session()?;

        // Execute the commands.
        channel.exec(&compiled_commands)?;

        // Read the output.
        let mut output = String::new();
        channel.read_to_string(&mut output)?;

        channel.wait_eof()?;
        channel.wait_close()?;
        channel.close()?;

        // Return the output.
        Ok(output)
    }

    /// Sends a directory recursively via SCP.
    ///
    /// # Arguments
    ///
    /// * `local_path` - The local path.
    /// * `remote_path` - The remote path.
    ///
    /// # Examples
    ///
    /// ```
    /// let sbs = Sbs::new(session); // Your SBS instance.
    ///
    /// let local_path = Path::new("/path/to/local_dir");
    /// let remote_path = Path::new("/path/to/remote_dir");
    ///
    /// sbs.send_directory(&local_path, &remote_path).unwrap();
    /// ```
    pub fn send_directory(&self, local_path: &Path, remote_path: &Path) -> Result<(), Error> {
        // Make sure the local path exists.
        if !local_path.exists() {
            return Err(Error::new(ErrorKind::NotFound, format!("The local path '{}' does not exist!", local_path.display())));
        }

        let sftp_session = self.session.sftp()?;

        // Make sure the remote path exists.
        match sftp_session.stat(remote_path) {
            Ok(stat) => {
                if !stat.is_dir() {
                    return Err(Error::new(ErrorKind::InvalidInput, format!("The remote path '{}' is not a directory!", remote_path.display())));
                }
            }
            Err(_) => {
                eprintln!("The remote path '{}' does not exist, creating it...", remote_path.display());

                Self::make_dirs(&sftp_session, remote_path);
            }
        }

        // Iterate over the local directory.
        for entry in local_path.read_dir()? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Send the directory recursively.
                self.send_directory(&path, &remote_path.join(entry.file_name()))?;
            } else {
                // Send the file.
                let mut remote_file = self.session.scp_send(
                    &remote_path.join(entry.file_name()),
                    0o755, // Read, write, execute by owner.
                    path.metadata()?.len(),
                    None,
                )?;

                let mut local_file = File::open(&path)?;
                io::copy(&mut local_file, &mut remote_file)?;

                remote_file.flush()?;
            }
        }

        Ok(())
    }

    fn make_dirs(sftp_session: &Sftp, remote_path: &Path) {
        let mut path = PathBuf::new();

        for component in remote_path.components() {
            path.push(component.as_os_str());

            match sftp_session.stat(&path) {
                Ok(stat) => {
                    if !stat.is_dir() {
                        panic!("The remote path '{}' is not a directory!", path.display());
                    }
                }
                Err(_) => {
                    sftp_session.mkdir(&path, 0o755).unwrap();
                }
            }
        }
    }

    /// Receives a directory recursively via SCP.
    ///
    /// # Arguments
    ///
    /// * `local_path` - The local path.
    /// * `remote_path` - The remote path.
    ///
    /// # Examples
    ///
    /// ```
    /// let sbs = Sbs::new(session); // Your SBS instance.
    ///
    /// let local_path = Path::new("/path/to/local_dir");
    /// let remote_path = Path::new("/path/to/remote_dir");
    ///
    /// sbs.receive_directory(&local_path, &remote_path).unwrap();
    /// ```
    pub fn receive_directory(&self, local_path: &Path, remote_path: &Path) -> Result<(), Error> {
        // Create the local directory.
        std::fs::create_dir_all(local_path)?;

        // Retrieve the directory contents.
        let remote_files = self.session.sftp()?.readdir(remote_path)?;

        // Iterate over the remote files.
        for remote_file in remote_files {
            let path_buf = remote_file.0;
            let file_stat = remote_file.1;

            let remote_filename = match path_buf.file_name() {
                Some(filename) => filename,
                None => continue,
            };

            let remote_file_path = remote_path.join(remote_filename);
            let local_file_path = local_path.join(remote_filename);

            if file_stat.is_dir() {
                // Create the corresponding local subdirectory.
                std::fs::create_dir_all(&local_file_path)?;

                // Receive the subdirectory recursively.
                self.receive_directory(&local_file_path, &remote_file_path)?;
            } else {
                // Receive the file.
                let remote_file = self.session.scp_recv(&remote_file_path)?;
                let mut local_file = File::create(&local_file_path)?;

                let mut channel = remote_file.0;
                io::copy(&mut channel, &mut local_file)?;

                local_file.flush()?;
            }
        }

        Ok(())
    }
}
