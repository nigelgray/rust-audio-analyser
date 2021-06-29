@echo off
REM Run this from the virtualenv

REM Copy across the generated lib file
cp target/debug/rust_audio_tester.dll python_src/rust_audio_tester.pyd

REM Run the Kivy script to start the app
python python_src/audio_tester.py
