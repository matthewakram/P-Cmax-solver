from matplotlib.pyplot import *
import matplotlib.pyplot as plt
import matplotlib


matplotlib.rcParams['figure.figsize'] = [2.8,2.8]
matplotlib.rcParams['font.family'] = 'serif'
matplotlib.rcParams['font.size'] = 8
plt.rcParams['text.usetex'] = True
subplots_adjust(bottom=0.14)
subplots_adjust(left=0.19)
subplots_adjust(right=0.98)
subplots_adjust(top=0.97)

files = [
    ["./complete_class_instances_hj.txt", "./complete_franca_frangioni_hj.txt", "./complete_lawrenko_hj.txt"],
    ["./complete_class_instances_hj_inter.txt", "./complete_franca_frangioni_hj_inter.txt", "./complete_lawrenko_hj_inter.txt"],
    ["./complete_class_instances_hj_base.txt", "./complete_franca_frangioni_hj_base.txt", "./complete_lawrenko_hj_base.txt"],
]

names = [ r'\textbf{HJFUR}', r'\textbf{HJINTER}' , r'\textbf{HJBASE}']


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
ax.set_ylim([4000, 4550])  
ax.set_xlim([0, 205])  
plt.savefig('./plots/HJ.pdf')
plt.show()
