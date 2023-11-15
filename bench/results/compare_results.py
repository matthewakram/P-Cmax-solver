import sys


file1 = sys.argv[1]
file2 = sys.argv[2]

with open(file1) as f1:
    with open(file2) as f2:
        lines1 = f1.readlines()
        lines2 = f2.readlines()
        for line in lines1:
            line = line.split()
            line2 = [x for x in lines2 if x.startswith(line[0])]
            if len(line2) == 0 :
                continue
            line2 = line2[0].split()
            if line2 [2] != line[2]:
                print("found error " + str(line))