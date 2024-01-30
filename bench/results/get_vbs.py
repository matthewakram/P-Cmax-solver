import sys

first_file = sys.argv[1]
second_file = sys.argv[2]
out_file = sys.argv[3]

first_sols = {}
with open(first_file) as f1:
    lines1 = f1.readlines()
    for line in lines1:
        line = line.strip().split(" ")
        first_sols[line[0]] = line[1:]

second_sols = {}
with open(second_file) as f2:
    lines2 = f2.readlines()
    for line in lines2:
        line = line.strip().split(" ")
        second_sols[line[0]] = line[1:]

out = ""
for key, val in first_sols.items():
    time1 = float(val[0])
    out += key
    out += " "
    if key in second_sols and time1 > float(second_sols[key][0]):
        out += " ".join(second_sols[key])
    else:
        out += " ".join(val)
    out += "\n"
    
for key, val in second_sols.items():
    if not key in first_sols :
        out += key 
        out += " "
        out += " ".join(val)
        out += "\n"

with open(out_file, 'w') as f:
    f.write(out)