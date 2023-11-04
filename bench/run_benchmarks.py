import os
import shlex
import subprocess
from subprocess import PIPE, STDOUT, check_output
import sys
from threading import Timer
import time

directory = os.fsencode( sys.argv[1])

def run(cmd, timeout_sec=60):
    process = subprocess.Popen(cmd, stdout=subprocess.PIPE)
    timer = Timer(timeout_sec, lambda : (process.terminate(), print("FAILED TIMEOUT" + " ".join(cmd))))
    out = b""
    try:
        timer.start()
        for c in iter(lambda: process.stdout.read(1), b""):
            out += c
    finally:
        timer.cancel()
        return out.decode("ascii")

options = input("what options would you like to have: ")
options = options.strip()
options_cpy = options
output = ""
options = options.split(" ")
file_num = 1
files = [os.fsdecode(x) for x in os.listdir(directory)]
files = [x for x in files if x.endswith(".txt")]
files.sort()
for filename in files:
    if "NU" in filename:
        continue
    
    with open(sys.argv[1] + "/" + filename, "r") as f1:
        if file_num == 245:
            pass
        sys.stdout.write("\rFile number: %i" % file_num)
        sys.stdout.flush()
        start = time.time()
        cmd = ['./target/release/p_cmax_solver',sys.argv[1] + "/" + filename] + options
        out = ""
        out = run(cmd)
        end = time.time()
        if "solution found" in out:
            place = out.find("solution found ")
            result = out[place + 15:].split()[0]
            finished = end - start
            line = f1.readline()
            line = line.split()
            n = line[2]
            m = line[3]
            num_unsat = int(str(out.count("UNSAT")))
            num_sat = int(str(out.count("SAT")))
            if "useful" in out:
                print(out+"\n")
            if True:
                num_sat = num_sat - num_unsat
                output += (""+ str(n)+"_"+str(m) +"_" + str(file_num) +" " + "x " + str(finished) + " " + result +"\n")
        else:
            print(out+"\n" + filename +"\n")
        file_num += 1
        
        subprocess.Popen(["killall", "kissat"], stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT)
     
with open("./bench/results/result_"+ options_cpy.replace(" ", "") +".txt", "w") as f :
    f.write(output)
