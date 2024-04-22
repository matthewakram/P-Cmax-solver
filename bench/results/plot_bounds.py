import matplotlib
import matplotlib.pyplot as plt

matplotlib.rcParams['figure.figsize'] = [5, 3.6]
matplotlib.rcParams['font.family'] = 'serif'
plt.rcParams['text.usetex'] = True
files = [
    ["./bound_berndt_trivial.txt", "./bound_franca_frangioni_trivial.txt", "./bound_lawrenko_trivial.txt", "./bound_real_cnf_trivial.txt", "./bound_real_graph_trivial.txt", "./bound_real_planted_trivial.txt", "./bound_real_rt_anni_trivial.txt", "./bound_real_rt_huebner_trivial.txt", "./bound_real_rt_laupichler_trivial.txt", "./bound_real_rt_lehmann_trivial.txt", "./bound_real_rt_schreiber_trivial.txt"],
    ["./bound_berndt_sss_bound_strengthening.txt", "./bound_franca_frangioni_sss_bound_strengthening.txt", "./bound_lawrenko_sss_bound_strengthening.txt", "./bound_real_cnf_sss_bound_strengthening.txt", "./bound_real_graph_sss_bound_strengthening.txt", "./bound_real_planted_sss_bound_strengthening.txt", "./bound_real_rt_anni_sss_bound_strengthening.txt", "./bound_real_rt_laupichler_sss_bound_strengthening.txt", "./bound_real_rt_huebner_sss_bound_strengthening.txt", "./bound_real_rt_lehmann_sss_bound_strengthening.txt", "./bound_real_rt_schreiber_sss_bound_strengthening.txt"],
    ["./bound_berndt_lifting_weak.txt", "./bound_franca_frangioni_lifting_weak.txt", "./bound_lawrenko_lifting_weak.txt", "./bound_real_cnf_lifting_weak.txt", "./bound_real_graph_lifting_weak.txt", "./bound_real_planted_lifting_weak.txt", "./bound_real_rt_anni_lifting_weak.txt", "./bound_real_rt_huebner_lifting_weak.txt", "./bound_real_rt_laupichler_lifting_weak.txt" ,"./bound_real_rt_lehmann_lifting_weak.txt", "./bound_real_rt_schreiber_lifting_weak.txt"],
    ["./bound_berndt_lifting.txt", "./bound_franca_frangioni_lifting.txt", "./bound_lawrenko_lifting.txt", "./bound_real_cnf_lifting.txt", "./bound_real_graph_lifting.txt", "./bound_real_planted_lifting.txt", "./bound_real_rt_anni_lifting.txt", "./bound_real_rt_huebner_lifting.txt", "./bound_real_rt_laupichler_lifting.txt" ,"./bound_real_rt_lehmann_lifting.txt", "./bound_real_rt_schreiber_lifting.txt"],
    ["./bound_berndt_ssss.txt", "./bound_franca_frangioni_ssss.txt", "./bound_lawrenko_ssss.txt", "./bound_real_cnf_ssss.txt", "./bound_real_graph_ssss.txt", "./bound_real_planted_ssss.txt", "./bound_real_rt_anni_ssss.txt", "./bound_real_rt_huebner_ssss.txt", "./bound_real_rt_laupichler_ssss.txt" , "./bound_real_rt_lehmann_ssss.txt", "./bound_real_rt_schreiber_ssss.txt"],
    ["./bound_berndt_mss.txt", "./bound_franca_frangioni_mss.txt", "./bound_lawrenko_mss.txt", "./bound_real_cnf_mss.txt", "./bound_real_graph_mss.txt", "./bound_real_planted_mss.txt", "./bound_real_rt_anni_mss.txt", "./bound_real_rt_huebner_mss.txt", "./bound_real_rt_laupichler_mss.txt" , "./bound_real_rt_lehmann_mss.txt", "./bound_real_rt_schreiber_mss.txt"],
    ["./bound_berndt_lptpp.txt", "./bound_franca_frangioni_lptpp.txt", "./bound_lawrenko_lptpp.txt", "./bound_real_cnf_lptpp.txt", "./bound_real_graph_lptpp.txt", "./bound_real_planted_lptpp.txt", "./bound_real_rt_anni_lptpp.txt", "./bound_real_rt_huebner_lptpp.txt","./bound_real_rt_laupichler_lptpp.txt" ,"./bound_real_rt_lehmann_lptpp.txt", "./bound_real_rt_schreiber_lptpp.txt"],
    ["./bound_berndt_lptp.txt", "./bound_franca_frangioni_lptp.txt", "./bound_lawrenko_lptp.txt", "./bound_real_cnf_lptp.txt", "./bound_real_graph_lptp.txt", "./bound_real_planted_lptp.txt", "./bound_real_rt_anni_lptp.txt", "./bound_real_rt_huebner_lptp.txt", "./bound_real_rt_laupichler_lptp.txt", "./bound_real_rt_lehmann_lptp.txt", "./bound_real_rt_schreiber_lptp.txt"],
    ["./bound_berndt_lpt.txt", "./bound_franca_frangioni_lpt.txt", "./bound_lawrenko_lpt.txt", "./bound_real_cnf_lpt.txt", "./bound_real_graph_lpt.txt", "./bound_real_planted_lpt.txt", "./bound_real_rt_anni_lpt.txt", "./bound_real_rt_huebner_lpt.txt", "./bound_real_rt_laupichler_lpt.txt", "./bound_real_rt_lehmann_lpt.txt", "./bound_real_rt_schreiber_lpt.txt"],
]
names = [r'\textbf{TV}', r'\textbf{LSSS}', r'\textbf{L}', r"""\underline{\textbf{L'}}""", r'\underline{\textbf{SSSS}}',r'\textbf{MSS}', r'\underline{\textbf{LPT\#}}', r'\underline{\textbf{LPT++}}', r'\textbf{LPT}']
colours = ["#32C8D8", "#059025", "#BA15B5", "#AFD80C", "#E50843", "#000000", "#737673", "#419AFF", "#FF5B41"]

num_lower_bounds = 4

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

x = list(range(1,len(sorted_bounds)+1))
for i in range(len(sorted_bounds[0])):
    y = []
    for file_bounds in sorted_bounds:
        y.append(file_bounds[i])
    y.sort()
    y.append(2)
    
    if i == 0:
        x.append(len(y))
    
    if i %3== 1:
        style= '-'
    elif i %3== 2:
        style = '--'
    else:
        style = 'dotted'
    plt.plot(y,x, label=names[i], linestyle=style, color=colours[i])

plt.legend(loc="lower right")

#plt.xscale("log")
#plt.yscale("log")
plt.xlabel('MDB')
plt.ylabel("Cumulative Bounded Instances")
ax = plt.gca()
ax.set_xlim([0.99, 1.26]) 
ax.set_ylim([8000, 16800])
plt.savefig('./plots/bound_plot.pdf')  
plt.show()
