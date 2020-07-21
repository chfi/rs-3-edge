GFA 3-Edge-Connectivity
==============================


Finds the 3-edge-connected components of a bridgeless graph in defined
in GFA. Output is one component per line, as a space-delimited list of
GFA segment names.

## Usage

```bash
$ ./three-edge-connected --help
three-edge-connected 0.1.0
Finds the 3-edge-connected components in a graph. Input must be a bridgeless graph in the GFA format. Output is a list
of 3-edge-connected components, one per line, as space-delimited lists of segment names

USAGE:
    three-edge-connected [FLAGS] [OPTIONS] --in-file <in-file> -s

FLAGS:
    -h, --help       Prints help information
    -s               If true, read input GFA on stdin
    -V, --version    Prints version information

OPTIONS:
    -i, --in-file <in-file>      GFA file to use, must be present if not reading from stdin
    -o, --out-file <out-file>    Output file. If empty, writes on stdout


$ ./three-edge-connected -i some.gfa -o output
$ ./three-edge-connected -i some.gfa -s > output
```
