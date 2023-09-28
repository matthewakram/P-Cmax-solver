#!/bin/bash

function gen() {
    filename="p_cmax-n${1}-m${2}-maxsize${3}-seed${4}.txt"
    echo "Generating $filename ..."
    python3 generate-single-benchmark.py $1 $2 $3 $4 > $filename
    nb_generated=$((nb_generated+1))
}


nb_generated=0


# Basic tests (with job sizes in 1..10)

# one job, one machine
gen 1 1 10 $RANDOM
gen 1 1 10 $RANDOM
# two jobs, one machine
gen 2 1 10 $RANDOM
gen 2 1 10 $RANDOM
# one job, two machines
gen 1 2 10 $RANDOM
gen 1 2 10 $RANDOM
# two jobs, two machines
gen 2 2 10 $RANDOM
gen 2 2 10 $RANDOM
gen 2 2 10 $RANDOM


# Scaling benchmarks (tweak value ranges to your liking)

for n in 5 10 20 50 100; do
    for m in 3 5 10 20; do
        if [ $m -ge $n ]; then continue; fi
        for maxsize in 3 5 10 20 50; do
            gen $n $m $maxsize $RANDOM
        done
    done
done


echo "Generated $nb_generated benchmarks."
