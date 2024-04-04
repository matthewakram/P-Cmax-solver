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
    ["./vbs_class_e2e.txt","./vbs_franca_frangioni_e2e.txt","./vbs_lawrenko_e2e.txt"],
    ["./complete_class_instances_mehdi_nizar_prec_e2e.txt", "./complete_franca_frangioni_mehdi_nizar_prec_e2e.txt", "./complete_lawrenko_mehdi_nizar_prec_e2e.txt"],
    ["./complete_class_instances_mehdi_nizar_original_optimization_e2e.txt", "./complete_franca_frangioni_mehdi_nizar_optimization_e2e.txt", "./complete_lawrenko_mehdi_nizar_original_optimization_e2e.txt"],
    ["./complete_class_instances_compressed_branch_and_bound_e2e.txt", "./complete_franca_frangioni_compressed_branch_and_bound_e2e.txt", "./complete_lawrenko_compressed_branch_and_bound_e2e.txt"],
    ["./complete_class_instances_hj_e2e.txt", "./complete_franca_frangioni_hj_e2e.txt", "./complete_lawrenko_hj_e2e.txt"],
    ["./complete_class_instances_multi_e2e.txt", "./complete_franca_frangioni_multi_e2e.txt", "./complete_lawrenko_multi_e2e.txt"],
    ["./complete_class_instances_intercomp_e2e.txt", "./complete_franca_frangioni_intercomp_e2e.txt", "./complete_lawrenko_instances_intercomp_e2e.txt"],
    ["./complete_class_instances_binmerge_simp_e2e.txt", "./complete_franca_frangioni_instances_binmerge_simp_e2e.txt", "./complete_lawrenko_instances_binmerge_simp_e2e.txt"],
    ["./complete_class_instances_basic_e2e.txt", "./complete_franca_frangioni_basic_e2e.txt", "./complete_lawrenko_instances_basic_e2e.txt"]
]

names = [r'\textbf{VBS}', r'\textbf{ILP}', r'\textbf{ILPBASE}', r'\textbf{DMFUR}', r'\textbf{HJFUR}', r'\textbf{MULTI}' ,r'\textbf{BDDFUR}', r'\textbf{BINMERGE}', r'\textbf{ADDER}']
colours = ["#00D7D7", "#C00404", "#7D0101", "#0066ff", "#000066", "#6FFF00", "#61E100", "#459C03", "#306C02"]
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
ax.set_ylim([5200, 7340])  
ax.set_xlim([0.00001, 905]) 
plt.savefig('./plots/all.pdf')

plt.xscale("log")
plt.legend(loc="upper left")
# plt.yscale("log") 
plt.savefig('./plots/all_logarithmic.pdf')


plt.show()