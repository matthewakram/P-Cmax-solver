# P-Cmax-solver

A sat based solver for the P||Cmax problem (multi-processor scheduling for uniform processors).
Repository made for my Master's Thesis.

## Running the tests.

We provide a docker container to better facilitate running the tests.
Simply download this repo and run
```
docker build -t submission_image .
```
to build it, and then start it by running

```
docker run -it submission_image
```

This container does nothing by default, use the shell to start the tests you would like to run.

The tests can be found in src/test/complete.rs. To run them, simply select the algorithm you would like to run, copy the name of its test, and run
```cargo test -r <test_name> -- --nocapture --ignored```
It will iterate through the datasets and provide the results in the results folder.
The gurobi solver is not provided due to licencing, thus the ilp solution cannot be run without manually installing gurobi. If you wish to do so, follow the instructions on their website and the copy the gurobi_ci executable to the root directory of the project.

Once you have some results for multiple algorithms, you can plot them as a 1v1 graph as follows:
```python3 <path to file 1> <path to file 2>`-xlabel=<label for file 1> -ylabel=<label for file 2>```
This produces a 1v1 graph comparing the performance of the algorithms that produced these files. 
If for example, the first algorithm took 6 seconds to solve and instance, and the second algorithm took 20 seconds, this instance will be represented by a point in (20,6), representing that algorithm 1 outperformed algorithm 2 on this instance.