[![Python](https://img.shields.io/badge/python-3.6+-blue.svg)](https://www.python.org/downloads/) [![Rust](https://github.com/DavidSeyserGit/ABB-Klipper-Middleware/actions/workflows/rust.yml/badge.svg)](https://github.com/DavidSeyserGit/ABB-Klipper-Middleware/actions/workflows/rust.yml) [![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0) ![GitHub repo file or directory count](https://img.shields.io/github/directory-file-count/DavidSeyserGit/ABB-Klipper-Middleware?style=flat) ![GitHub Tag](https://img.shields.io/github/v/tag/DavidSeyserGit/ABB-Klipper-Middleware)


# ABB-Klipper-Bridge
A Rust crate for establishing a network connection with an ABB robot and forwarding data to the Moonraker API.

This crate simplifies the process of connecting to a TCP server, reading data, and sending it to a Moonraker instance.  It's designed to be lightweight and efficient, providing a building block for applications that need to interact with Moonraker over TCP.


## Installation

### Quick Start
```bash
git clone https://github.com/DavidSeyserGit/ABB-Klipper-Middleware.git
cd ABB-Klipper-Middleware
```

### Converter Install Using Makefile (recommended)
```bash
make converter
```
#### Usage

### Single File Processing
```bash
abb-converter /path/to/file.mod rapid
```

### Directory Processing
```bash
abb-converter /path/to/directory/ klipper
```

### Help
```bash
python3 converter.py --help
```

## Features

* **Network Connection Management:** Establishes and manages a persistent network connections.
* **Data Forwarding:** Reads data from the netwotk stream and forwards it to the Moonraker API.
* **Configurable:** Allows customization of network connection parameters and Moonraker API endpoint.
* **Error Handling:** Provides robust error handling for connection issues and API interactions.
* **Minimal Dependencies:** Keeps dependencies to a minimum for a smaller footprint.

## License and Attributions

This project is licensed under the GNU General Public License v3.0 (GPLv3).  See the [LICENSE](LICENSE) file for the full license text.

This project incorporates the following third-party libraries/crates:

* **[reqwest](https://github.com/seanmonstar/reqwest):** Licensed under the MIT License.  See the [MIT License](LICENSE-crate_name_1) file for the full license text.

* **[colored](https://github.com/colored-rs/colored):** Licensed under the MPL-2.0. See the [MPL-2.0 License](LICENSE-crate_name_2) file for the full license text.

* **[tokio](https://github.com/tokio-rs/tokio):** Licensed under the MIT. See the [MIT License](LICENSE-crate-name_3) file for the full license text.

## Explanation of Licenses

This project uses several open-source licenses.  Here's a brief overview:

* **GPLv3:** The GNU General Public License version 3.0 is a copyleft license. This means that if you distribute a modified version of this project, you must also distribute the source code of your modifications under the GPLv3.

* **MIT License:** The MIT License is a very permissive license that allows you to do almost anything with the code, as long as you include the original copyright notice and permission notice.

* **Apache License 2.0:** The Apache License 2.0 is another permissive license that is similar to the MIT License, but also includes a patent grant.

* **Mozilla Public License 2.0:** The MPL 2.0 is a file-level copyleft license.  It allows you to combine the MPL 2.0-licensed code with other code under other licenses, including the GPLv3, and distribute the combined work under the terms of the GPLv3.
