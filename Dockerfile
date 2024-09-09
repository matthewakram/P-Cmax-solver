FROM ubuntu:20.04

# Update default packages
RUN apt-get update

# Get Ubuntu packages
RUN apt-get install -y \
    build-essential \
    curl \
    git \
    python3 \
    unzip \
    wget

# Update new packages
RUN apt-get update

# Get Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN git clone https://github.com/matthewakram/P-Cmax-solver.git

WORKDIR /P-Cmax-solver/bench/class_instances/
RUN ./generate-benchmark-suite.sh

WORKDIR /P-Cmax-solver/bench/lawrenko/
RUN ./generate-benchmark-suite.sh

ENV RUST_MIN_STACK 200000000
WORKDIR /P-Cmax-solver/bench/franca_frangioni
RUN wget https://site.unibo.it/operations-research/en/research/library-of-codes-and-instances-1/cmax.zip/@@download/file/cmax.zip
RUN unzip ./cmax.zip -d ./unzipped
RUN unzip ./unzipped/PCmax_instances.zip -d ./
RUN mkdir ./standardised
WORKDIR /P-Cmax-solver
RUN python3 ./bench/franca_frangioni/transform.py

WORKDIR /P-Cmax-solver

# The tests can be found in src/test/complete.rs. To run them, simply select the algorithm you 
# would like to run, copy the name of its test, and run
# cargo test -r <test_name> -- --nocapture --ignored
# it will iterate through the datasets and provide the results in the results folder.
# The gurobi solver is not provided due to licencing, thus the ilp solution cannot be run
# without manually installing gurobi. If you wish to do so, follow the instructions on their 
# website and the copy the gurobi_ci executable to the root directory of the project. 

ENTRYPOINT ["sleep", "infinity"]