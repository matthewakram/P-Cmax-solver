import os
import shlex
import subprocess
from subprocess import PIPE, STDOUT, check_output
import sys
from threading import Timer
import time

directory = os.fsencode("./bench/class_instances")

def run(cmd, timeout_sec=20):
    process = subprocess.Popen(cmd, stdout=PIPE)
    timer = Timer(timeout_sec, lambda : (process.kill(), print("FAILED TIMEOUT")))
    try:
        timer.start()
        for c in iter(lambda: process.stdout.read(1), b""):
               sys.stdout.buffer.write(c)
    finally:
        timer.cancel()

start = time.time()
for file in os.listdir(directory):
     filename = os.fsdecode(file)
     if filename.endswith(".txt"):
         
         print(filename)
         cmd = ['./target/release/p_cmax_solver','./bench/class_instances/'+ filename]
         run(cmd)
         print("\n")
         
      
end = time.time()      

print("it took " + str(end - start) + " seconds to complete tests")

