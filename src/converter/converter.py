#!/usr/bin/env python3
"""
ABB Robot Code Converter - Python Version
Converts ABB robot .mod files to add socket communication functionality.
"""

import sys
import os
import re
import argparse
from pathlib import Path
from typing import Optional


class RobotConverter:
    """Converter class for processing ABB robot files."""
    
    def __init__(self, postprocessor: str = "rapid"):
        self.postprocessor = postprocessor
        
    def read_file(self, file_path: Path) -> str:
        """Read file contents and return as string."""
        try:
            with open(file_path, 'r', encoding='utf-8') as file:
                return file.read()
        except Exception as e:
            print(f"Error reading file '{file_path}': {e}", file=sys.stderr)
            raise
    
    def search_and_create_socket(self, contents: str, _postprocess: str) -> str:
        """Add socket variable declaration and initialization to the robot code."""
        # Check if socket code is already present
        if all(pattern in contents for pattern in [
            "VAR socketdev my_socket",
            "SocketCreate my_socket",
            "SocketConnect my_socket, \"10.0.0.10\", 1234"
        ]):
            return contents
        
        modified_contents = []
        module_found = False
        proc_found = False
        
        for line in contents.splitlines():
            modified_contents.append(line)
            
            # Add socket variable declaration after MODULE line
            if "MODULE" in line and not module_found:
                module_found = True
                modified_contents.append("VAR socketdev my_socket;")
            
            # Add socket creation and connection after PROC line
            if "PROC" in line and not proc_found:
                proc_found = True
                modified_contents.append("\tSocketCreate my_socket;")
                modified_contents.append("\tSocketConnect my_socket, \"10.0.0.10\", 1234;")
        
        if not module_found:
            return "Error: Could not find the MODULE line."
        if not proc_found:
            return "Error: Could not find the PROC main_RoboDK() line."
        
        return '\n'.join(modified_contents)
    
    def replace_call_extruder_with_socket_send(self, contents: str, postprocess: str) -> str:
        """Replace extruder commands with socket send commands."""
        new_contents = []
        
        extruder_pattern = re.compile(r"Extruder\s?(\d+)")
        
        for line in contents.splitlines():
            match = extruder_pattern.search(line)
            if match:
                number_str = match.group(1)
                factor = 100000.00 if postprocess == "rapid" else 1.00
                number = float(number_str) / factor
                new_contents.append(f"    SocketSend my_socket \\Str := \"E{number}\";")
            else:
                new_contents.append(line)

        return '\n'.join(new_contents)
    
    def replace_setrpm_with_socket_send(self, contents: str, postprocess: str) -> str:
        """Replace RPM commands with socket send commands."""
        if postprocess == "rapid":
            pattern = re.compile(r"SetRPM(\d+)")
        else:
            pattern = re.compile(r"SetRPM\s+(\d+)")
        
        new_contents = []
        
        for line in contents.splitlines():
            match = pattern.search(line)
            if match:
                number_str = match.group(1)
                number = float(number_str)
                new_contents.append(f"    SocketSend my_socket \\Str := \"F{number}\";")
            else:
                new_contents.append(line)
        
        return '\n'.join(new_contents)
    
    def replace_m_code_with_socket_send(self, contents: str, postprocess: str) -> str:
        """Replace M-code commands with socket send commands."""
        if postprocess == "rapid":
            pattern = re.compile(r"M_RunCode(\d+)")
        else:
            pattern = re.compile(r"M_RunCode\s+(\d+)")
        
        new_contents = []
        
        for line in contents.splitlines():
            match = pattern.search(line)
            if match:
                number_str = match.group(1)
                number = float(number_str)
                new_contents.append(f"    SocketSend my_socket \\Str := \"M{number}\";")
            else:
                new_contents.append(line)
        
        return '\n'.join(new_contents)
    
    def process_file(self, file_path: Path, postprocessor: str):
        """Process a single .mod file."""
        print(f"\033[92mProcessing File: {file_path}\033[0m")  # Green text
        
        # Check file extension
        if file_path.suffix != ".mod":
            print("\033[91mError: Only *.mod files are accepted\033[0m", file=sys.stderr)
            sys.exit(1)
        
        # Read and process file
        contents = self.read_file(file_path)
        
        # Apply all transformations
        contents = self.search_and_create_socket(contents, postprocessor)
        if contents.startswith("Error:"):
            print(f"\033[91m{contents}\033[0m", file=sys.stderr)
            return
            
        contents = self.replace_call_extruder_with_socket_send(contents, postprocessor)
        contents = self.replace_setrpm_with_socket_send(contents, postprocessor)
        contents = self.replace_m_code_with_socket_send(contents, postprocessor)
        
        # Write back to file
        with open(file_path, 'w', encoding='utf-8') as file:
            file.write(contents)
    
    def process_directory(self, directory_path: Path, postprocessor: str):
        """Process all .mod files in a directory."""
        for file_path in directory_path.iterdir():
            if file_path.is_file():
                if file_path.suffix == ".mod":
                    # Special handling for main.mod
                    contents = self.read_file(file_path)
                    
                    if file_path.name == "main.mod":
                        contents = self.search_and_create_socket(contents, postprocessor)
                        if contents.startswith("Error:"):
                            print(f"\033[91m{contents}\033[0m", file=sys.stderr)
                            continue
                    
                    contents = self.replace_call_extruder_with_socket_send(contents, postprocessor)
                    contents = self.replace_setrpm_with_socket_send(contents, postprocessor)
                    contents = self.replace_m_code_with_socket_send(contents, postprocessor)
                    
                    # Write back to file
                    with open(file_path, 'w', encoding='utf-8') as file:
                        file.write(contents)
                        
                    print(f"\033[92mProcessing File: {file_path}\033[0m")  # Green text
                else:
                    print("\033[91mError: Only *.mod files are accepted\033[0m", file=sys.stderr)
                    sys.exit(1)
            elif not file_path.name.startswith('.'):  # Skip hidden directories
                print(f"\033[91mError: File has no extension: {file_path}\033[0m", file=sys.stderr)


def main():
    """Main entry point for the converter."""
    parser = argparse.ArgumentParser(
        description="Convert ABB robot .mod files to add socket communication functionality",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python converter.py /path/to/file.mod rapid
  python converter.py /path/to/directory/ klipper
        """
    )
    
    parser.add_argument(
        "path",
        help="Path to .mod file or directory containing .mod files"
    )
    
    parser.add_argument(
        "postprocessor",
        help="Postprocessor type (e.g., 'rapid', 'klipper')"
    )
    
    args = parser.parse_args()
    
    # Validate path
    path = Path(args.path)
    if not path.exists():
        print(f"\033[91mError: Path '{path}' does not exist\033[0m", file=sys.stderr)
        sys.exit(1)
    
    # Create converter instance
    converter = RobotConverter(args.postprocessor)
    
    print(args.postprocessor)
    
    # Process file or directory
    try:
        if path.is_file():
            converter.process_file(path, args.postprocessor)
        elif path.is_dir():
            converter.process_directory(path, args.postprocessor)
        else:
            print(f"\033[91mError: '{path}' is neither a file nor a directory\033[0m", file=sys.stderr)
            sys.exit(1)
            
        print("\033[92mConversion completed successfully!\033[0m")
        
    except Exception as e:
        print(f"\033[91mError during conversion: {e}\033[0m", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main() 