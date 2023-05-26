The goal is to not make our host computer compile the code.

## How to use
```bash
$ sbs /path/to/config.json
```

## Example
```bash
$ sbs hello_world.toml
```

The above command starts SBS and compiles the code in the `hello_world` directory, on the server and sends it back to the local `target/release` directory.

## Configuration
The configuration file is a TOML file. The following is an example of a configuration file:
```toml
[ssh]
host = "localhost"
port = 22
username = "root"
password = "root"

[compilation]
local_project_root = "/home/user/hello_world" # The path to the project on your local machine from the root of the project.
remote_project_root = "/compilation/hello_world" # The path to the project on the remote machine from the root of the project.
output_directory = "target/release" # The directory where the compiled binary is located relative to the project root.

[[commands]]
command = "cd /compilation/hello_world"
description = "Change directory to the project root."
execute_after_compilation = false

[[commands]]
command = "cargo build --release"
description = "Build the project."
execute_after_compilation = false
```
