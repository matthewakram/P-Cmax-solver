#!/bin/bash

function gen() {
    filename="p_cmax-n${1}-m${2}-minsize${3}-maxsize${4}-seed${5}.txt"
    echo "Generating $filename ..."
    python3 generate-single-benchmark.py $1 $2 $3 $4 $5 > $filename
    nb_generated=$((nb_generated+1))
}


nb_generated=0



# Scaling benchmarks (tweak value ranges to your liking)

for i in 1 2; do
    for n in 10 20 50 100; do
        for m in 2 3 5 8 10; do
            if [ $m -ge $n ]; then continue; fi
            for minsize in 1 50 100; do
                for maxsize in 20 50 100 200 800; do
                    if [ $minsize -ge $maxsize ]; then continue; fi
                    gen $n $m $minsize $maxsize $RANDOM
                done
            done
        done
    done
done


echo "Generated $nb_generated benchmarks."
