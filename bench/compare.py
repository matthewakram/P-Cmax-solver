import os
import subprocess
import sys
from threading import Timer
import time

directory = os.fsencode("./bench/class_instances")

def run(cmd, timeout_sec=30):
    process = subprocess.Popen(cmd, stdout=subprocess.PIPE)
    timer = Timer(timeout_sec, lambda : (process.terminate(), print("FAILED TIMEOUT")))
    out = b""
    try:
        timer.start()
        for c in iter(lambda: process.stdout.read(1), b""):
            out += c
    finally:
        if b"SAT" in out:
            print(out.decode("ascii"))
        timer.cancel()
 
start = time.time()
for file in os.listdir(directory):
     filename = os.fsdecode(file)
     if filename.endswith(".txt"):
         
         print(filename)
         cmd = ['./target/release/p_cmax_solver','./bench/class_instances/'+ filename, "-fur"]
         run(cmd)
         print("\n")
         print(filename)
         cmd = ['./target/release/p_cmax_solver','./bench/class_instances/'+ filename, "-furlite"]
         run(cmd)
         print("\n")
         
      
end = time.time()      

print("it took " + str(end - start) + " seconds to complete tests")

