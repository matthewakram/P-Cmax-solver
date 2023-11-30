import sys


file1 = sys.argv[1]
stat_num = int(sys.argv[2])

total = 0.0
with open(file1) as f1:
    lines1 = f1.readlines()
    for line in lines1:
        line = line.split()
        stat = float(line[stat_num])
        total += stat

print(total)