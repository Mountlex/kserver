# kserver

[![Build Status](https://travis-ci.org/Mountlex/kserver.svg?branch=master)](https://travis-ci.org/Mountlex/kserver)

## BrightKite

Run the script `download_data.sh` to download and preprocess the BrightKite-dataset.

## Simulation

The following commands were used to generate the results in the paper.

**Attention: These simulation runs require a lot of RAM if they are executed on many parallel threads. To manually control the number of threads set the RAYON_NUM_THREADS environmental variable. We executed our results on a server with 64 cores and 2 TB of RAM.**

k = 2:

```bash
cargo run --release -- -l 1000 -p 10 -b 1 -k 2 -s100 -m5 -o bk_k2_.csv load_instances bk -d data kserver --lambdas 11
cargo run --release -- -l 1000 -p 10 -b 1 -k 2 -s100 -m5 -o bk_k2_lazy.csv load_instances bk -d data kserver --lambdas 11 --lazy
```

k = 10:

```bash
cargo run --release -- -l 1000 -p 10 -b 2 -k 10 -s100 -m5 -o bk_k10_.csv load_instances bk -d data kserver --lambdas 11
cargo run --release -- -l 1000 -p 10 -b 2 -k 10 -s100 -m5 -o bk_k10_lazy.csv load_instances bk -d data kserver --lambdas 11 --lazy
```

k = 50:

```bash
cargo run --release -- -l 1000 -p 10 -b 3 -k 50 -s100 -m5 -o bk_k50_.csv load_instances bk -d data kserver --lambdas 11
cargo run --release -- -l 1000 -p 10 -b 3 -k 50 -s100 -m5 -o bk_k50_lazy.csv load_instances bk -d data kserver --lambdas 11 --lazy
```

## Results

The `csv`-files which where used to generate the figures in the paper are located at `paper_results`. You can plot them using the `plot` script:

```bash
./plot paper_results/bk_k2.csv -b 1 -l 0.0 0.1 0.5
./plot paper_results/bk_k2_lazy.csv -b 1 -l 0.0 0.1 0.5
./plot paper_results/bk_k10.csv -b 2 -l 0.0 0.1 0.5
./plot paper_results/bk_k10_lazy.csv -b 2 -l 0.0 0.1 0.5
./plot paper_results/bk_k50.csv -b 3 -l 0.0 0.1 0.5
./plot paper_results/bk_k50_lazy.csv -b 3 -l 0.0 0.1 0.5
```
