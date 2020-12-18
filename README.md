# kserver

[![Build Status](https://travis-ci.org/Mountlex/kserver.svg?branch=master)](https://travis-ci.org/Mountlex/kserver)

## BrightKite

To download and preprocess the dataset, use the script `download_data.sh`.

## Simulation

The following commands were used to generate the results in the paper.

k = 2:

```bash
cargo run --release -- -l 1000 -p 20 -b 10 -k 2 -s500 -m5 -o bk_k2_l1000_p20_b10_lmb11.csv load_instances bk -d data kserver --lambdas 11
```

k = 5:

```bash
cargo run --release -- -l 1000 -p 20 -b 32 -k 5 -s500 -m5 -o bk_k10_l1000_p20_b32_lmb11.csv load_instances bk -d data kserver --lambdas 11
```

k = 10:

```bash
cargo run --release -- -l 1000 -p 20 -b 75 -k 10 -s500 -m5 -o bk_k10_l1000_p20_b75_lmb11.csv load_instances bk -d data kserver --lambdas 11
```

k = 100:

```bash
cargo run --release -- -l 1000 -p 20 -b 700 -k 100 -s500 -m5 -o bk_k100_l1000_p20_b700_lmb11.csv load_instances bk -d data kserver --lambdas 11
```

## Results

The `csv`-files which where used to generate the figures in the paper are located at `paper_results`. You can plot them using the `plot` script, e.g.

```bash
./plot paper_results/bk_k2_l1000_p20_b10_lmb11.csv -b10 -l 0.0 0.1 0.5 1.0
```
