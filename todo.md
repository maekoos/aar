# To Do

- [ ] Update README and TODO
- [ ] Handle parsing errors - use the `ParserError`

- [ ] Keep track of each instance's superclasses
- [ ] Printing values
  - `Foo@2c7b84de`?
  - [Object.toString()](https://stackoverflow.com/questions/29140402/how-do-i-print-my-java-object-without-getting-sometype2f92e0f4)
- [ ] Make sure `<clinit>` is run before creating the first instance **depending** on it - Static variables + Inheritance

- [ ] Documentation
- [ ] Better executable (something like: `./aar main.dex --main Main.hello_world --args 10 15`)
- [ ] Multiple input DEX-files

- [ ] Remove python as a dependency?
  - Makefile
  - `build.rs`?
- [ ] Remove android as a dependency
  -  Is this even possible?

- [ ] Test environment (run in both java and dalvik to compare output)
- [ ] Unit tests?

## Exceptions
[Errors vs exceptions](https://www.tutorialspoint.com/java/java_exceptions.htm#stickyparent:~:text=Errors%20are%20abnormal%20conditions%20that%20happen,Normally%2C%20programs%20cannot%20recover%20from%20errors.)

- Keep track of subclasses
- Does the order of the handlers matter?
- Rename handlers and/or exceptions type from something like `Ljava/lang/ArithmeticException;` to our naming convention

## java.lang, etc
- Find a better way to write the standard library than what we do in the `std_env` mod
  - Especially necessary when we get to writing the android-specific built in libraries
- Better strings + test performance

# dexparser
*Since aar heavily depends on `dexparser` and it seems to be quite untested and immature, we may need to update/rewrite some parts of it. Here are a few todos for the dexparser crate that would benefit the aar project.* 

- What is, and how do we use `TryItem.code_units`?
  - Doesn't seem to be actual code units
  - Error in the library?
  - **For now:** Replace `code_units` with the original raw `start_addr` and `insn_count` (and use `git="https://github.com/maekoos/dexparser.git"`)

- Too much exposed on `Code`?
  - Both `tries` and `handlers` - should only expose `handlers` or use a `Rc<EncodedCatchHandler>`?
