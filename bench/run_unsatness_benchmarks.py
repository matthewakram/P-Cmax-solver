import os
import shlex
import subprocess
from subprocess import PIPE, STDOUT, check_output
import sys
from threading import Timer
import time

directory = os.fsencode( sys.argv[1])
timout = 20

def run(cmd, timeout_sec=20):
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
output2 = []
for filename in files:
    distance = int(filename.split("_")[1][:-4])
    instance_num = int(filename.split("_")[0])
    with open(sys.argv[1] + "/" + filename, "r") as f1:
        sys.stdout.write("\rFile number: %i" % file_num)
        sys.stdout.flush()
        start = time.time()
        cmd = ['./kissat',sys.argv[1] + "/" + filename]
        out = ""
        out = run(cmd, timout)
        end = time.time()
        if "s UNSATISFIABLE" in out:
            unsat = 0
            sat = 0
            
            
            finished = end - start
            output2.append([distance, instance_num, file_num])
            output += (str(distance)+"_"+str(distance) +"_"+ str(instance_num) +"_"+ str(file_num) +" " + "x " + str(finished) +"\n")
        file_num += 1

with open(out_file, "w") as f:
    f.write(output)

biggest_instance = max([x[1] for x in output2])

with open(out_file + "a", 'w') as f:
    f.write("\n".join([str(x[0]) + "_" + str(x[0]) + "_" +str(x[1]) + "_"+ str(x[2]) + " " + "x "+str(timout*x[1] / biggest_instance) for x in output2])) 