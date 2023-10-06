import os
import shlex
import subprocess
from subprocess import PIPE, STDOUT, check_output
import sys
from threading import Timer
import time

directory = os.fsencode( sys.argv[1])

def run(cmd, timeout_sec=30):
    process = subprocess.Popen(cmd, stdout=subprocess.PIPE)
    timer = Timer(timeout_sec, lambda : (process.terminate(), print("FAILED TIMEOUT")))
    out = b""
    try:
        timer.start()
        for c in iter(lambda: process.stdout.read(1), b""):
            out += c
    finally:
        timer.cancel()
        return out.decode("ascii")

input_dir = sys.argv[1]
out_file = sys.argv[2]

file_num = 1
files = [os.fsdecode(x) for x in os.listdir(directory)]
files = [x for x in files if x.endswith(".txt")]
files.sort()
output = ""
for filename in files:
    with open(sys.argv[1] + "/" + filename, "r") as f1:
        sys.stdout.write("\rFile number: %i" % file_num)
        sys.stdout.flush()
        start = time.time()
        cmd = ['./kissat',sys.argv[1] + "/" + filename]
        out = ""
        out = run(cmd)
        end = time.time()
        if "s UNSATISFIABLE" in out or "s SATISFIABLE" in out:
            unsat = 0
            sat = 0
            if "s UNSATISFIABLE" in out:
                unsat = 1
            else:
                sat = 1
            finished = end - start
            output += (str(sat)+"_"+str(unsat) +"_" + str(file_num) +" " + "x " + str(finished) +"\n")
        file_num += 1

with open(out_file, "w") as f:
    f.write(output)