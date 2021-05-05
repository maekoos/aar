# AAR
*Alternative APK Runtime* is a prototype runtime for APKs.

## Current state
AAR is currently just a proof of concept. It's able to run some DEX-files, although with a very limited set of features, and far from the whole dalvik instruction set.

In the future, AAR will hopefully be able to run apps on desktop computers, mobile devices, and maybe even on the web. Someday, maybe.

## Try it yourself!
### Dependencies
- Rust and Cargo
- Android SDK (d8 and dexdump)
- Java
- Python 3 (development only)


### Usage
First, make sure the `android SDK/build tools` directory is in your path. Then, open your preferred terminal in this directory and run
```bash
make
```

This runs the aar and you should see some output from the `resources/MyCode/MyCode.java` file, which was compiled into dex and then interpreted. In the `./out` directory you will find the IR and CFA of the input file.

## Project structure
A simple summary of the most important files and folders.
```
- resources/    : JAVA test file + Makefile for DEX compilation
- src/          : AAR source code
    - parser/   : Parsing and preparing DEX for codegen
    - codegen/  : Where the parsed dex-files become interpreted code
    - lib.rs    : Main library file with the process function
    - main.rs   : Basic program to run the DEX file in resources
```

## License
[Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)