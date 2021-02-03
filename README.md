# kserver

[![Build Status](https://travis-ci.org/Mountlex/kserver.svg?branch=master)](https://travis-ci.org/Mountlex/kserver)

## BrightKite

Run the script `download_data.sh` to download and preprocess the BrightKite-dataset.

## Simulation

The following commands were used to generate the results in the paper.

**Attention: These simulation runs require a lot of RAM if they are executed on many parallel threads. To manually control the number of threads set the RAYON_NUM_THREADS environmental variable. We executed our results on a server with 64 cores and 2 TB of RAM.**

k = 2:

```bash
cargo run --release -- -l 1000 -p 10 -b 28 -k 2 -s500 -m5 -o bk_k2.csv load_instances bk -d data kserver --lambdas 11
```

k = 5:

```bash
cargo run --release -- -l 1000 -p 10 -b 75 -k 5 -s500 -m5 -o bk_k10.csv load_instances bk -d data kserver --lambdas 11
```

k = 10:

```bash
cargo run --release -- -l 1000 -p 10 -b 170 -k 10 -s500 -m5 -o bk_k10.csv load_instances bk -d data kserver --lambdas 11
```

k = 100:

```bash
cargo run --release -- -l 1000 -p 10 -b 1700 -k 100 -s500 -m5 -o bk_k100.csv load_instances bk -d data kserver --lambdas 11
```

## Results

The `csv`-files which where used to generate the figures in the paper are located at `paper_results`. You can plot them using the `plot` script, e.g.

```bash
./plot paper_results/bk_k2_l1000_p10_b28.csv -b28 -l 0.0 0.1 0.5 1.0
```
