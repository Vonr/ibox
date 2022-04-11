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
        Specify the border characters.
        Default: ┌─┐│└┘
    -l=LENGTH
        Specify the max length of the input.
        Default: 8
    -p=X,Y
        Specify the position of the top left corner of the box.
        Default: current cursor position
    -c
        Center the box on the screen.
    -h
        Print this help message and exit.
```
