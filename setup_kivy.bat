REM Create virtualenv
python -m virtualenv kivy_venv

REM Enter the virtualenv
kivy_venv/Scripts/activate

REM Install Kivy
python -m pip install kivy[base]
