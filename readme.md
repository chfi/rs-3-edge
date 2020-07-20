GFA 3-Edge-Connectivity
==============================


## Usage

```bash
$ git clone https://github.com/chfi/rs-3-edge.git
$ cd rs-3-edge
$ cargo build --release
$ ./target/release/three-edge-connected some.gfa
```

Outputs the 3-edge-connected components of the given GFA file to
stdout, with each line consisting of a tab-delimited list of the
segments of one of the components.
