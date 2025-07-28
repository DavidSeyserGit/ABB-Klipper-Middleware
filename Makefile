.PHONY: help install install-dev test test-verbose lint format type-check clean build upload-test upload

# Default target
help:
	@echo "ABB Klipper Middleware - Development Commands"
	@echo ""
	@echo "Available commands:"
	@echo "  install        Install the package"
	@echo "  test           Run tests"
	@echo "  test-verbose   Run tests with verbose output"
	@echo "  lint           Run linting (flake8)"
	@echo "  format         Format code with black"
	@echo "  type-check     Run type checking with mypy"
	@echo "  clean          Clean build artifacts"
	@echo "  build          Build distribution packages"
	@echo "  upload-test    Upload to Test PyPI"
	@echo "  upload         Upload to PyPI"
	@echo ""

# Installation
converter:
	pip install -e src/converter

bridge:
	cargo build --bin bridge --release

all: 
	pip install -e src/converter
	cargo build --bin bridge

# Testing
test:
	python3 -m pytest

test-verbose:
	python3 -m pytest -v --tb=long

test-coverage:
	python3 -m pytest --cov=converter --cov-report=html --cov-report=term

# Code quality
lint:
	flake8 converter.py tests/

format:
	black converter.py tests/

format-check:
	black --check converter.py tests/

type-check:
	mypy converter.py

# Quality check all
check: format-check lint type-check test

# Cleanup
clean:
	rm -rf build/
	rm -rf target/
	rm -rf dist/
	rm -rf *.egg-info/
	rm -rf __pycache__/
	rm -rf tests/__pycache__/
	rm -rf .pytest_cache/
	rm -rf .mypy_cache/
	rm -rf .coverage
	rm -rf htmlcov/
	find . -type f -name "*.pyc" -delete
	find . -type d -name "__pycache__" -delete

# Building and distribution
build: clean
	python3 -m build

upload-test: build
	python3 -m twine upload --repository testpypi dist/*

upload: build
	python3 -m twine upload dist/*

# Development workflow
dev-setup: install-dev
	@echo "Development environment set up successfully!"
	@echo "Run 'make test' to run tests"
	@echo "Run 'make check' to run all quality checks"

# Quick development check
quick-check:
	python3 -m py_compile converter.py
	python3 converter.py --help 