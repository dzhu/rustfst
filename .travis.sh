#!/bin/sh

set -ex

./build_openfst.sh
./run_openfst.sh

cargo build --all
cargo test --all
cargo check --benches --all # running benches on travis is useless
cargo doc --all

virtualenv venv3 -p python3.6
. venv3/bin/activate
pip install -e rustfst-python-bench
# Run benches on a small FST to check that the script is working fine.
python3 rustfst-python-bench/rustfst_python_bench/bench_all.py rustfst-tests-data/fst_003/raw_vector.fst report.md
