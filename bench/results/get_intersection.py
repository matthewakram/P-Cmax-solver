import sys

first_file = sys.argv[1]
second_file = sys.argv[2]
out_file = sys.argv[3]
e_number = int(sys.argv[4])

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
    val1 = int(val[e_number-1])
    if key in second_sols and val1 != 0:
        val2 = int(second_sols[key][e_number-1])
        out += key
        out += " "
        out += str(val2 / val1)
        out += "\n"

with open(out_file, 'w') as f:
    f.write(out)