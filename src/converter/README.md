[![Python](https://img.shields.io/badge/python-3.6+-blue.svg)](https://www.python.org/downloads/) [![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0) ![GitHub repo file or directory count](https://img.shields.io/github/directory-file-count/DavidSeyserGit/ABB-Klipper-Middleware?style=flat) ![GitHub Tag](https://img.shields.io/github/v/tag/DavidSeyserGit/ABB-Klipper-Middleware)

# ABB Klipper Middleware

A Python tool for converting ABB robot `.mod` files to add socket communication functionality for seamless integration with Klipper/Moonraker 3D printing systems.

## Overview

This project provides a converter that processes ABB robot program files (`.mod` format) and automatically adds TCP socket communication capabilities. The converted files can then communicate with Klipper through the Moonraker API, enabling advanced 3D printing workflows with industrial robot precision.

## Features

- **🔌 Socket Integration**: Automatically adds socket variable declaration and initialization to robot code
- **📡 Command Translation**: Converts robot commands to socket send operations:
  - `Extruder` commands → `E` commands via socket
  - `SetRPM` commands → `F` commands via socket  
  - `M_RunCode` commands → `M` commands via socket
- **⚙️ Postprocessor Support**: Handles different postprocessor types (`rapid`, `klipper`, etc.)
- **📁 Batch Processing**: Can process individual files or entire directories
- **🎨 Rich Output**: Comprehensive error checking and colored terminal output
- **🐍 Pure Python**: No external dependencies - uses only standard library

## Requirements

- **Python 3.6+** (recommended: Python 3.8+)
- No external dependencies required

## Installation

### Quick Start
```bash
git clone https://github.com/DavidSeyserGit/ABB-Klipper-Middleware.git
cd ABB-Klipper-Middleware
chmod +x converter.py
```

### Using pip (recommended)
```bash
pip install -e .
```

## Usage

### Single File Processing
```bash
python3 converter.py /path/to/file.mod rapid
```

### Directory Processing
```bash
python3 converter.py /path/to/directory/ klipper
```

### After installation with pip
```bash
abb-converter /path/to/file.mod rapid
```

### Help
```bash
python3 converter.py --help
```

## Supported Postprocessors

- **`rapid`**: For ABB RAPID postprocessor format
- **`klipper`**: For Klipper integration format
- Custom postprocessor types supported

## How It Works

1. **Socket Setup**: Adds socket variable declaration after `MODULE` line and socket creation/connection after `PROC` line
2. **Command Translation**:
   - Extruder commands are converted to socket send commands with `E` prefix
   - RPM commands are converted to socket send commands with `F` prefix
   - M-code commands are converted to socket send commands with `M` prefix
3. **Smart Processing**: For non-rapid postprocessors, calculates E-values based on MoveL commands and extruder speed

## Socket Configuration

The converter configures socket connections to:
- **IP**: `10.0.0.10`
- **Port**: `1234`

These can be modified in the source code as needed for your setup.

## Development

### Setting up development environment
```bash
git clone https://github.com/DavidSeyserGit/ABB-Klipper-Middleware.git
cd ABB-Klipper-Middleware
pip install -e ".[dev]"
```

### Running tests
```bash
pytest
```

### Code formatting
```bash
black converter.py
```

### Type checking
```bash
mypy converter.py
```

## Examples

### Converting a single robot program
```bash
python3 converter.py robot_program.mod rapid
```

### Processing all .mod files in a directory
```bash
python3 converter.py ./robot_programs/ klipper
```

### Expected output
```
rapid
🟢 Processing File: robot_program.mod
🟢 Conversion completed successfully!
```

## File Processing Logic

- Only processes files with `.mod` extension
- For directories: special handling for `main.mod` files (gets socket setup)
- Preserves original file structure and formatting
- Adds socket communications while maintaining robot code functionality

## Error Handling

The converter provides intuitive colored output:
- 🟢 **Green**: Success messages and file processing status
- 🔴 **Red**: Error messages and warnings
- 🟡 **Yellow**: Usage information

## Migration from Rust Version

This Python version maintains full compatibility with the original Rust implementation:
- ✅ Same command-line interface
- ✅ Identical processing logic
- ✅ Same output format
- ✅ Compatible file handling
- ✅ All original features preserved

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License and Attributions

This project is licensed under the GNU General Public License v3.0 (GPLv3). See the [LICENSE](LICENSE) file for the full license text.

### Third-Party Licenses

This project incorporates the following third-party libraries:

- **Python Standard Library**: Licensed under the Python Software Foundation License

## License Overview

- **GPLv3**: The GNU General Public License version 3.0 is a copyleft license. This means that if you distribute a modified version of this project, you must also distribute the source code of your modifications under the GPLv3.

## Support

- 📁 [Issues](https://github.com/DavidSeyserGit/ABB-Klipper-Middleware/issues)
- 📖 [Documentation](https://github.com/DavidSeyserGit/ABB-Klipper-Middleware)
- 💬 [Discussions](https://github.com/DavidSeyserGit/ABB-Klipper-Middleware/discussions)

---

**Keywords**: ABB Robot, Klipper, 3D Printing, Manufacturing Automation, Industrial Robotics, Python