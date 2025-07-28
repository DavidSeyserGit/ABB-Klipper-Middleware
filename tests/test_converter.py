#!/usr/bin/env python3
"""
Tests for the ABB Robot Code Converter
"""

import unittest
import tempfile
import os
from pathlib import Path
import sys

# Add the parent directory to the path so we can import converter
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from converter import RobotConverter


class TestRobotConverter(unittest.TestCase):
    """Test cases for RobotConverter class."""

    def setUp(self):
        """Set up test fixtures."""
        self.converter = RobotConverter("rapid")
        
    def test_socket_creation(self):
        """Test socket creation functionality."""
        test_content = """MODULE TestModule
PROC main_RoboDK()
    ! Test procedure
ENDPROC
ENDMODULE"""
        
        result = self.converter.search_and_create_socket(test_content, "rapid")
        
        # Check that socket components were added
        self.assertIn("VAR socketdev my_socket;", result)
        self.assertIn("SocketCreate my_socket;", result)
        self.assertIn('SocketConnect my_socket, "10.0.0.10", 1234;', result)
        
    def test_socket_creation_already_exists(self):
        """Test that socket creation is skipped if already present."""
        test_content = """MODULE TestModule
VAR socketdev my_socket;
PROC main_RoboDK()
    SocketCreate my_socket;
    SocketConnect my_socket, "10.0.0.10", 1234;
    ! Test procedure
ENDPROC
ENDMODULE"""
        
        result = self.converter.search_and_create_socket(test_content, "rapid")
        
        # Should return original content unchanged
        self.assertEqual(result, test_content)
        
    def test_extruder_replacement_rapid(self):
        """Test extruder command replacement for rapid postprocessor."""
        test_content = """Extruder123456
Some other line
Extruder789012"""
        
        result = self.converter.replace_call_extruder_with_socket_send(test_content, "rapid")
        
        # Check that extruder commands were replaced
        self.assertIn('SocketSend my_socket \\Str := "E1.23456";', result)
        self.assertIn('SocketSend my_socket \\Str := "E7.89012";', result)
        self.assertIn("Some other line", result)
        
    def test_setrpm_replacement(self):
        """Test SetRPM command replacement."""
        test_content = """SetRPM1500
Some other line
SetRPM2000"""
        
        result = self.converter.replace_setrpm_with_socket_send(test_content, "rapid")
        
        # Check that RPM commands were replaced
        self.assertIn('SocketSend my_socket \\Str := "F1500.0";', result)
        self.assertIn('SocketSend my_socket \\Str := "F2000.0";', result)
        self.assertIn("Some other line", result)
        
    def test_m_code_replacement(self):
        """Test M-code command replacement."""
        test_content = """M_RunCode104
Some other line
M_RunCode109"""
        
        result = self.converter.replace_m_code_with_socket_send(test_content, "rapid")
        
        # Check that M-codes were replaced
        self.assertIn('SocketSend my_socket \\Str := "M104.0";', result)
        self.assertIn('SocketSend my_socket \\Str := "M109.0";', result)
        self.assertIn("Some other line", result)
        
    def test_error_no_module(self):
        """Test error handling when MODULE line is missing."""
        test_content = """PROC main_RoboDK()
    ! Test procedure
ENDPROC"""
        
        result = self.converter.search_and_create_socket(test_content, "rapid")
        
        self.assertEqual(result, "Error: Could not find the MODULE line.")
        
    def test_error_no_proc(self):
        """Test error handling when PROC line is missing."""
        test_content = """MODULE TestModule
    ! Test module
ENDMODULE"""
        
        result = self.converter.search_and_create_socket(test_content, "rapid")
        
        self.assertEqual(result, "Error: Could not find the PROC main_RoboDK() line.")


class TestConverterIntegration(unittest.TestCase):
    """Integration tests for file processing."""
    
    def test_file_processing(self):
        """Test processing a complete .mod file."""
        test_content = """MODULE TestModule
PROC main_RoboDK()
    Extruder123456
    SetRPM1500
    M_RunCode104
ENDPROC
ENDMODULE"""
        
        # Create temporary file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.mod', delete=False) as tmp:
            tmp.write(test_content)
            tmp_path = Path(tmp.name)
            
        try:
            converter = RobotConverter("rapid")
            converter.process_file(tmp_path, "rapid")
            
            # Read back the processed file
            with open(tmp_path, 'r') as f:
                result = f.read()
                
            # Check that all transformations were applied
            self.assertIn("VAR socketdev my_socket;", result)
            self.assertIn("SocketCreate my_socket;", result)
            self.assertIn('SocketConnect my_socket, "10.0.0.10", 1234;', result)
            self.assertIn('SocketSend my_socket \\Str := "E1.23456";', result)
            self.assertIn('SocketSend my_socket \\Str := "F1500.0";', result)
            self.assertIn('SocketSend my_socket \\Str := "M104.0";', result)
            
        finally:
            # Clean up
            if tmp_path.exists():
                tmp_path.unlink()


if __name__ == '__main__':
    unittest.main() 