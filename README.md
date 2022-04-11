# ibox
[![Crates.io](https://img.shields.io/crates/v/ibox)](https://crates.io/crates/ibox)

### Simple input box drawing command line utility.

### Usage
```
Usage: ibox [OPTION]... [QUERY]...
Search for QUERY in FILES.
Example:
    ibox -l=24 'Title' 'Context' 'Question: ?>'
Options:
    -b=BORDER
        Specify the border characters or presets.
        Presets: single (default), double, thick, curved
        Default: ┌─┐│└┘
    -l=LENGTH
        Specify the added length of the input space after the longest line.
        Default: 8
    -p=X,Y
        Specify the position of the top left corner of the box.
        Default: current cursor position
    -c
        Center the box on the screen.
    -s
        Makes the box stretch to the terminal's sides.
    -h
        Print this help message and exit.
```
