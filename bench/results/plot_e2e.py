import matplotlib.pyplot as plt
from matplotlib.pyplot import *
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
    ["./complete_cdsm.txt"],
    ["./complete_cdsm_irrelevance.txt"],
    ["./complete_cdsm_fur.txt"],
    ["./complete_cdsm_inter.txt"],
    ["./complete_cdsm_last_size.txt"],
    ["./complete_cdsm_base.txt"],
]

names = [r'\textbf{CDSM}', r'\textbf{IRRELEVANT}', r'\textbf{FUR}', r'\textbf{INTER}', r'\textbf{LAST}', r'\textbf{BASE}']
colours = ["#00D7D7", "#C00404", "#7D0101", "#0066ff", "#000066", "#6FFF00"]
styles= ["-", ":", "-", "-", "-", "-", "-", "-", "-"]


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
    y.append(1000)
    x.append(len(x))
    if i % 2 == 1:
        style = "-"
    else:
        style = "--"
    plt.plot(y,x, label=names[i], color=colours[i], linestyle=styles[i])

plt.legend(loc="lower right")

plt.xlabel("Time (s)")
plt.ylabel("Cumulative Solved Instances")
ax = plt.gca()
ax.set_ylim([15000, 17000])  
ax.set_xlim([0.00001, 505]) 
plt.savefig('./plots/all.pdf')

# plt.xscale("log")
# plt.legend(loc="upper left")
# plt.yscale("log") 
# plt.savefig('./plots/all_logarithmic.pdf')


plt.show()