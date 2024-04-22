import glob

files = glob.glob('./bench/results/bound*ssss.txt')
files +=  glob.glob('./bench/results/bound*lifting.txt')

results = dict()

for file in files:
    with open(file) as f:
        lines = f.readlines()
        for line in lines:
            if line == "":
                continue
            splitted = line.split()
            if not splitted[0] in results:
                results[splitted[0]] = (int(splitted[2]), int(splitted[3]))
            else:
                low, high = results[splitted[0]]
                low = max(low, int(splitted[2]))
                high = min(high, int(splitted[3]))
                results[splitted[0]] = (low,high)

num_solved = 0

for (low, high) in results.values():
    if low == high:
        num_solved += 1

print(num_solved)       