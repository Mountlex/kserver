# kserver

[![Build Status](https://travis-ci.org/Mountlex/kserver.svg?branch=master)](https://travis-ci.org/Mountlex/kserver)

#### bk k=2

```bash
cargo run --release -- -l 600 -p 30 -b 2 -k 2 -o bk_k2_l300_p30_b2.csv load_instances bk -d data kserver --lambdas 6
cargo run --release -- -l 300 -p 15 -b 2 -k 2 -o bk_k2_l300_p15_b2.csv load_instances bk -d data kserver --lambdas 6
```

#### bk k=50

```bash
cargo run --release -- -l 300 -p 30 -b 120.0 -k 50 -o bk_k50_l300_p30_b120.csv load_instances bk -d data kserver --lambdas 6
cargo run --release -- -l 600 -p 30 -b 240.0 -k 50 -o bk_k50_l600_p30_b240.csv load_instances bk -d data kserver --lambdas 6
```
