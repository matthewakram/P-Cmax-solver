import matplotlib.pyplot as plt

files = [
    ["./complete_class_instances_compressed_branch_and_bound.txt", "./complete_franca_frangioni_compressed_branch_and_bound.txt", "./complete_lawrenko_compressed_branch_and_bound.txt"],
    ["./complete_class_instances_compressed_branch_and_bound_inter.txt", "./complete_franca_frangioni_compressed_branch_and_bound_inter.txt", "./complete_lawrenko_compressed_branch_and_bound_inter.txt"],
    ["./complete_class_instances_compressed_branch_and_bound_basic.txt", "./complete_franca_frangioni_compressed_branch_and_bound_basic.txt", "./complete_lawrenko_compressed_branch_and_bound_basic.txt"],
]

names = ["DMFUR", "DMINTER", "DMBASE"]


bounds = []
i = 0
for bound_files in files:
    all_bounds = {}
    for file in bound_files:
        with open(file) as f:
            for line in f.readlines():
                line = line.split(" ")
                name = line[0]
                bound = float(line[1])
                all_bounds[name] = bound
    bounds.append([x for x in all_bounds.values()])
    i+=1



#sorted_bounds.sort(key=lambda x: max(x))

for i in range(len(bounds)):
    y = []
    for b in bounds[i]:
        y.append(b)
    y.sort()
    x = list(range(len(y)))
    plt.plot(y,x, label=names[i])

plt.legend(loc="lower right")

#plt.xscale("log")
#plt.yscale("log")
plt.xlabel("Time (s)")
plt.ylabel("Cumulative Solved Instances")
ax = plt.gca()
ax.set_ylim([3000, 4800])  
plt.savefig('./plots/DM.pdf')
plt.show()
