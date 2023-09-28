import os

directory = os.fsencode("./bench/class_instances")

for file in os.listdir(directory):
     filename = os.fsdecode(file)
     if filename.endswith(".txt"): 
        stream = os.popen('./target/release/p_cmax_solver ./bench/class_instances/'+ filename)
        output = stream.read()
        print(output)