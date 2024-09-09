# P-Cmax-solver

A sat based solver for the P||Cmax problem (multi-processor scheduling for uniform processors).
Repository made for my Master's Thesis.

## Running the tests.

We provide a docker container to better facilitate running the tests.
Simply download this repo and run
```
docker build -t submission_image .
```
to build it, and then start it from the docker gui.
This container does nothing by default, use the shell in the docker gui to start the tests you would like to run.

The tests can be found in src/test/complete.rs. To run them, simply select the algorithm you would like to run, copy the name of its test, and run
```cargo test -r <test_name> -- --nocapture --ignored```
It will iterate through the datasets and provide the results in the results folder.
The gurobi solver is not provided due to licencing, thus the ilp solution cannot be run without manually installing gurobi. If you wish to do so, follow the instructions on their website and the copy the gurobi_ci executable to the root directory of the project.
