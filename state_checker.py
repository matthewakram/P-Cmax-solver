
states = set()


num_repeated = 0
total = 0
with open("log.log") as f:
    for l in f.readlines():
        total += 1
        l = l[1:-2]
        l = l.split(", ")
        l = [int(x) for x in l]
        l = tuple(l)
        if l in states:
            num_repeated += 1
        else:
            states.add(l)
    print(str(num_repeated) + "/" + str(total) + " states are repeated (" + str(num_repeated * 100 / total) + "%)")
        