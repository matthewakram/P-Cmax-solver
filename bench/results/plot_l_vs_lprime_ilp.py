from matplotlib.pyplot import *
import matplotlib.pyplot as plt
import matplotlib


matplotlib.rcParams['figure.figsize'] = [5.6,2.8]
matplotlib.rcParams['font.family'] = 'serif'
matplotlib.rcParams['font.size'] = 8
plt.rcParams['text.usetex'] = True
subplots_adjust(bottom=0.14)
subplots_adjust(top=0.97)

files = [
    ["./with_l_ilp_class.txt", "./with_l_ilp_franca.txt", "./with_l_ilp_lawrenko.txt", "./with_l_ilp_real_cnf.txt", "./with_l_ilp_real_running_times.txt"],
    ["./with_l_prime_ilp_class.txt", "./with_l_prime_ilp_franca.txt", "./with_l_prime_ilp_lawrenko.txt", "./with_l_prime_ilp_real_cnf.txt", "./with_l_prime_ilp_real_running_times.txt"],
]

names = [ r"with \textbf{L}", r"with \textbf{L'}"]


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
    x = list(range(1,len(y)+1))
    y.append(300)
    x.append(len(x))
    plt.plot(y,x, label=names[i])

plt.legend(loc="lower right")

#plt.xscale("log")
#plt.yscale("log")
plt.xlabel("Time (s)")
plt.ylabel("Cumulative Solved Instances")
ax = plt.gca()
ax.set_ylim([5000, 9000])  
ax.set_xlim([0, 205])
plt.savefig('./plots/l_vs_lprime.pdf')
plt.show()
