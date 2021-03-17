# AAR
*Alternative APK Runtime* is a prototype runtime for APKs.

## Current state
AAR is currently just a proof of concept. It's able to run some DEX-files, although with a very limited set of features. It can, for example, not handle static functions and only supports a very small portion of the DEX instruction set.

no longer under active development ~~In the future, AAR will hopefully be able to run apps on desktop computers as well as mobile devices (and maybe even on the web), in a way which protects the users privacy and lets them view and control exactly what information they share with the application.~~

## Try it yourself!
### Dependencies
- Rust and Cargo
- A C-compiler (e.g. GCC)
- Android SDK

*AAR is developed, and has only been tested on, MacOs. However, it should run on other UNIX distributions, and Windows, with minimal modifications.*
### Usage
First, make sure the `android SDK/build tools` is in your path. Open your preferred terminal in the root directory and run
```bash
make
```

This runs the AAR generator and in the out directory you should be able to find out.c and out.h.
To compile the generated source files, go to `out/` and run `make`. This will output an `app.app` executable file.


***Important:** This is currently just a proof of concept. The generated code will not run because no `java.*` packages have been implemented. With that said, it will run if you comment out `CLASS_3____constructor(...)` (line 10) in out.c* 

## Project structure
A simple summary of the most important files and folders.
```
- libruntime/   : The runtime utilities used by the app
- out/          : Output directory + Makefile and an example main.c
- resources/    : JAVA test file + Makefile for DEX compilation
- src/          : AAR source code
    - generate/ : The C generator
    - parser/   : Parsing and preparing DEX before generating
    - lib.rs    : Main library file with parse_and_generate function
    - main.rs   : Basic program to generate C from the DEX file in resources
```

## Resources
- [Android documentation: Dalvik bytecode](https://source.android.com/devices/tech/dalvik/dalvik-bytecode)

## License
[Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
