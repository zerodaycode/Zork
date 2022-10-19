""" _summary_

    Provides the configuration for build the project with `Pyinstaller` as an executable
"""

from setuptools import setup, find_packages

setup(
    name='zork-manager',
    version='0.3.0',
    license='MIT',
    author="Alex Vergara",
    author_email='pyzyryab@tutanota.com',
    packages=find_packages('zork'),
    package_dir={'': 'zork'},
    url='https://github.com/zerodaycode/Zork',
    keywords='C++ project manager and build system',
    install_requires=[
		'pyinstaller'
	],
)
