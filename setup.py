import os
from pathlib import Path
from setuptools import setup, Extension

import pybind11
from pybind11.setup_helpers import Pybind11Extension, build_ext

src_dir = str(Path(__file__).resolve().parent)

sources = [
    os.path.join(src_dir, 'middleout.cpp'),
    os.path.join(src_dir, 'scalar.cpp'),
]

ext_modules = [
    Pybind11Extension(
        'middleout',
        sources,
        include_dirs=[
            pybind11.get_include(),
            src_dir,
        ],
        language='c++',
    ),
]

setup(
    name='middleout',
    version='0.0.1',
    author='ExData, Inc.',
    description='A pybind11 module for middleout compression',
    ext_modules=ext_modules,
)
