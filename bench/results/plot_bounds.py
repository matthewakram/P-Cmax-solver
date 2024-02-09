import matplotlib.pyplot as plt

files = [
    ["./bound_class_pigeon.txt", "./bound_franca_pigeon.txt", "./bound_lawrenko_pigeon.txt"],
    ["./bound_class_middle_jobs.txt", "./bound_franca_middle_jobs.txt", "./bound_lawrenko_middle_jobs.txt"],
    ["./bound_class_max_job_size.txt", "./bound_franca_max_job_size.txt", "./bound_lawrenko_max_job_size.txt"],
    ["./bound_class_sss_bound_strengthening.txt", "./bound_franca_sss_bound_strengthening.txt", "./bound_lawrenko_sss_bound_strengthening.txt"],
    ["./bound_class_lifting_weak.txt", "./bound_franca_lifting_weak.txt", "./bound_lawrenko_lifting_weak.txt"],
    ["./bound_class_lifting.txt", "./bound_franca_lifting.txt", "./bound_lawrenko_lifting.txt"],
    ["./bound_class_mss.txt", "./bound_franca_mss.txt", "./bound_lawrenko_mss.txt"],
    ["./bound_class_lptpp.txt", "./bound_franca_lptpp.txt", "./bound_lawrenko_lptpp.txt"],
    ["./bound_class_lptp.txt", "./bound_franca_lptp.txt", "./bound_lawrenko_lptp.txt"],
    ["./bound_class_lpt.txt", "./bound_franca_lpt.txt", "./bound_lawrenko_lpt.txt"]
]

names = ["TV", "LSSS", "L", "L'", "MSS", "LPT#", "LPT++", "LPT"]

num_lower_bounds = 6

bounds = []
i = 0
for bound_files in files:
    all_bounds = {}
    for file in bound_files:
        with open(file) as f:
            for line in f.readlines():
                line = line.split(" ")
                name = line[0]
                bound = 0
                if i < num_lower_bounds:
                    bound = int(line[2])
                else:
                    bound = int(line[3])
                all_bounds[name] = bound
    bounds.append(all_bounds)
    i+=1

for i in range(2):
    for file in bounds[0] :
        if bounds[0][file] < bounds[1][file]:
            bounds[0][file] = bounds[1][file]
    bounds.pop(1)
num_lower_bounds -= 2

sorted_bounds = []
for file in bounds[0]:
    bounds_for_file = []
    for bound in bounds:
        bounds_for_file.append(bound[file])
    sorted_bounds.append(bounds_for_file)

for i in range(len(sorted_bounds)):
    min_bound = min(sorted_bounds[i])
    max_bound = max(sorted_bounds[i])
    if max_bound != 0:
        for j in range(len(sorted_bounds[i])):
            sorted_bounds[i][j] /= min_bound


#sorted_bounds.sort(key=lambda x: max(x))

x = list(range(len(sorted_bounds)))
for i in range(len(sorted_bounds[0])):
    y = []
    for file_bounds in sorted_bounds:
        y.append(file_bounds[i])
    y.sort()
    if names[i] == "L'":
        print(y)
    plt.plot(y,x, label=names[i])

plt.legend(loc="lower right")

#plt.xscale("log")
#plt.yscale("log")
plt.xlabel("MDB")
plt.ylabel("Cumulative Bounded Instances")
plt.savefig('./plots/bound_plot.pdf')  
plt.show()
