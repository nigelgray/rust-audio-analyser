@echo off
REM Copy across the generated lib file
cp target/debug/rust_audio_tester.dll python_src/rust_audio_tester.pyd

REM Run the test script to try out the python lib
python python_src/test.py
