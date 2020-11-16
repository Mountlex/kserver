# kserver

[![Build Status](https://travis-ci.org/Mountlex/kserver.svg?branch=master)](https://travis-ci.org/Mountlex/kserver)

#### bk k=2

```
cargo run --release -- -l 1000 -p 100 -b 4.0 -k 2 -o test.csv load_instances bk -d data kserver --lambdas 6
```

#### Sampled k=2

```
cargo run --release -- -l 300 -p 22 -b 0.25 -k 2 -o sample.csv load_instances bk -d data kserver --lambdas 6
```

#### Sampled k=10

```
cargo run --release -- -l 300 -p 27 -b 3.0 -k 10 -o sample.csv sample 1000 kserver --lambdas 6
```
