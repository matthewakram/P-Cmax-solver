import sys
import statistics

first_file = sys.argv[1]
second_file = sys.argv[2]

first_sols = {}
with open(first_file) as f1:
    lines1 = f1.readlines()
    for line in lines1:
        line = line.strip().split(" ")
        first_sols[line[0]] = float(line[16])

second_sols = {}
with open(second_file) as f2:
    lines2 = f2.readlines()
    for line in lines2:
        line = line.strip().split(" ")
        second_sols[line[0]] = float(line[16])


diff_factors = []

for key, val in first_sols.items():
    if key in second_sols:
        val2 = second_sols[key]
        if val2 != 0.0:
            diff_factors.append(val / val2)

print("geometric mean: " + str(float(statistics.geometric_mean(diff_factors))))
print("max: " + str(float(max(diff_factors))))
print("min: " + str(float(min(diff_factors))))