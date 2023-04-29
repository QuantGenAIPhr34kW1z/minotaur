SHELL := /bin/sh

FORTRAN_DIR := src/fortran
RUST_DIR    := src/rust

BIN := target/release/minotaur

.PHONY: build fortran rust clean run sweep plots test

build: fortran rust

fortran:
	cd $(FORTRAN_DIR) && fpm build --profile release

rust:
	cd $(RUST_DIR) && cargo build --release

run: build
	cd $(RUST_DIR) && cargo run --release -- \
		--config ../../configs/baseline.toml \
		--out ../../results/out_baseline.csv

sweep: build
	cd $(RUST_DIR) && cargo run --release -- \
		--config ../../configs/sweep.toml \
		--out ../../results/out_sweep.csv \
		--mode sweep

plots:
	gnuplot plots/plot_fuel_vs_bpr.gp
	gnuplot plots/plot_iter_vs_regime.gp
	gnuplot plots/plot_t4_margin.gp

test: build
	sh tests/smoke.sh

clean:
	cd $(FORTRAN_DIR) && fpm clean || true
	cd $(RUST_DIR) && cargo clean || true
	rm -f results/out_baseline.csv results/out_sweep.csv
	rm -f results/fig_*.png
