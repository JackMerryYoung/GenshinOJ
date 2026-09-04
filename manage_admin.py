#!/usr/bin/env python3
"""Compatibility entry point for the role-based administrator management script."""

import os
import pathlib
import sys


script = pathlib.Path(__file__).with_name("manage_admin.sh")
os.execvp("bash", ["bash", str(script), *sys.argv[1:]])
