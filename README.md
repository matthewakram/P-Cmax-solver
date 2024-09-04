# P-Cmax-solver

A sat based solver for the P||Cmax problem (multi-processor scheduling for uniform processors).
Repository made for my Master's Thesis.

# Unfortunately it is not worth for me to document this, if you are interested in using it do not hesitate to contact me.
If you just want to run benchmark experiments, the tests can be run via
```
cargo test -r --test-threads=<num threads> --ignored <test name> -- --nocapture
```
and the tests themselves can be located under src/tests
