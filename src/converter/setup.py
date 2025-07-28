#!/usr/bin/env python3
"""
Setup script for ABB Klipper Middleware
"""

from setuptools import setup, find_packages
from pathlib import Path

# Read the README file
this_directory = Path(__file__).parent
long_description = (this_directory / "README.md").read_text(encoding='utf-8')

setup(
    name="abb-klipper-middleware",
    version="0.1.0",
    author="David Seyser",
    description="ABB Robot Code Converter for Klipper Integration",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/DavidSeyserGit/ABB-Klipper-Middleware",
    packages=find_packages(),
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Manufacturing",
        "Topic :: Scientific/Engineering",
        "License :: OSI Approved :: GNU General Public License v3 (GPLv3)",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.6",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
    ],
    python_requires=">=3.6",
    install_requires=[
        # No external dependencies - uses only standard library
    ],
    extras_require={
        "dev": [
            "pytest>=6.0",
            "black",
            "flake8",
            "mypy",
        ],
    },
    entry_points={
        "console_scripts": [
            "abb-converter=converter:main",
        ],
    },
    scripts=["converter.py"],
    keywords="abb robot klipper 3d-printing manufacturing automation",
    project_urls={
        "Bug Reports": "https://github.com/DavidSeyserGit/ABB-Klipper-Middleware/issues",
        "Source": "https://github.com/DavidSeyserGit/ABB-Klipper-Middleware",
    },
) 