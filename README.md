[![Rust](https://github.com/DavidSeyserGit/ABB-Klipper-Middleware/actions/workflows/rust.yml/badge.svg)](https://github.com/DavidSeyserGit/ABB-Klipper-Middleware/actions/workflows/rust.yml) [![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)![GitHub repo file or directory count](https://img.shields.io/github/directory-file-count/DavidSeyserGit/ABB-Klipper-Middleware?style=for-the-badge)


# ABB-Klipper-Bridge
A Rust crate for establishing a network connection with an ABB robot and forwarding data to the Moonraker API.

This crate simplifies the process of connecting to a TCP server, reading data, and sending it to a Moonraker instance.  It's designed to be lightweight and efficient, providing a building block for applications that need to interact with Moonraker over TCP.

## Features

* **Network Connection Management:** Establishes and manages a persistent network connections.
* **Data Forwarding:** Reads data from the netwotk stream and forwards it to the Moonraker API.
* **Configurable:** Allows customization of network connection parameters and Moonraker API endpoint.
* **Error Handling:** Provides robust error handling for connection issues and API interactions.
* **Minimal Dependencies:** Keeps dependencies to a minimum for a smaller footprint.
