# Python-Compiler

PYC is a Work-in-Progress compiled language based on Python's syntax.

## Build

```powershell
cmake -S . -B build
cmake --build build --config Debug
ctest --test-dir build -C Debug --output-on-failure
```
