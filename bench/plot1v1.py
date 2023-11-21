#!/usr/bin/env python3

import math
import sys

markers = ['.', 'v', 'x']
colors = ['#377eb8', '#ff7f00', '#e41a1c', '#f781bf', '#a65628', '#4daf4a', '#984ea3', '#999999', '#dede00', '#377eb8']

lim = 60


msize = 6
pltxsize = 5
pltysize = 5
xmin = None
xmax = None
ymin = None
ymax = None
y2 = None

files = []
runtime_maps = []
labels = []
xlabel = None
ylabel = None
min_val = lim
heading = ""
outfile = None
logscale = False
tickslist = None
legendright = False
legendbottom = False
legend_columns = 1
legend_offset_x = 0
legend_offset_y = 0
legend_spacing = 0.5
outfile_legend = None
powers_of_ten_ticks = False
stats = (0,0)

for arg in sys.argv[1:]:
    if arg.startswith("-l="):
        labels += [arg[3:]]
    elif arg.startswith("-h="):
        heading = arg[3:]
    elif arg.startswith("-size="):
        pltxsize = float(arg[6:])
        pltysize = float(arg[6:])
    elif arg.startswith("-xsize="):
        pltxsize = float(arg[7:])
    elif arg.startswith("-ysize="):
        pltysize = float(arg[7:])
    elif arg.startswith("-min="):
        border_lo = float(arg[5:])
    elif arg.startswith("-max="):
        border_hi = float(arg[5:])
    elif arg.startswith("-xlabel="):
        xlabel = arg[8:]
    elif arg.startswith("-ylabel="):
        ylabel = arg[8:]
    elif arg.startswith("-T="):
        lim = float(arg[3:])
        #out = lim * 1.35
        #border_hi = lim * 1.8
        min_val = lim
    elif arg.startswith("-y2="):
        y2 = float(arg[4:])
    elif arg.startswith("-legend-cols="):
        legend_columns = int(arg[len("-legend-cols="):])
    elif arg.startswith("-legend-offset-x="):
        legend_offset_x = float(arg[len("-legend-offset-x="):])
    elif arg.startswith("-legend-offset-y="):
        legend_offset_y = float(arg[len("-legend-offset-y="):])
    elif arg.startswith("-legend-spacing="):
        legend_spacing = float(arg[len("-legend-spacing="):])
    elif arg.startswith("-o="):
        outfile = arg[3:]
    elif arg.startswith("-ol="):
        outfile_legend = arg[len("-ol="):]
    elif arg.startswith("-logscale"):
        logscale = True
    elif arg.startswith("-ticks="):
        tickslist = arg[len("-ticks="):].split(",")
    elif arg.startswith("-markersize="):
        msize = float(arg[len("-markersize="):])
    elif arg.startswith("-potticks"):
        powers_of_ten_ticks = True
    elif arg.startswith("-stats="):
        split_string = arg[7:].split(",")
        stats = (int(split_string[0]), int(split_string[1]))
    else:
        files += [arg]

values = list()
statistics = list()

for arg in files:
    values_of_file = dict()
    statistics_of_file = dict()
    for line in open(arg, 'r').readlines():
        words = line.rstrip().split(" ")
        id = words[0]
        val = float(words[1])

        values_of_file[id] = val
        statistics_of_file[id] = [float(x) for x in words[2:]]
        
        if val > 0:
            min_val = min(min_val, val)
    values.append(values_of_file)
    statistics.append(statistics_of_file)

lim = max(list(values[0].values()) + list(values[1].values()))


out = lim+1
border_lo = 0.001
border_hi = lim+lim * 0.02

import matplotlib
if outfile:
    matplotlib.use('pdf')
matplotlib.rcParams['hatch.linewidth'] = 0.5  # previous pdf hatch linewidth
import matplotlib.pyplot as plt
from matplotlib import rc

sansfont = False
timesfont = False

rc('text', usetex=True)
if sansfont:
    matplotlib.rcParams['text.latex.preamble'] = [r'\usepackage[cm]{sfmath}']
    matplotlib.rcParams['font.family'] = 'sans-serif'
    matplotlib.rcParams['font.sans-serif'] = 'cm'
    #\renewcommand\familydefault{\sfdefault} 
else:
    rc('font', family='serif')
    if timesfont:
        rc('font', serif=['Times'])

out = math.exp((math.log(lim) + math.log(border_hi)) / 2)



margin = lim - out



fig, ax = plt.subplots(1, 1, figsize=(pltxsize, pltysize))
ax.set_box_aspect(1)

plt.ylim(min_val, border_hi)
plt.xlim(min_val, border_hi)
if logscale:
    plt.xscale("log")
    plt.yscale("log")

if powers_of_ten_ticks:
    power = -7
    while 10**power < min_val:
        power += 1
    tickpos = []
    ticklabel = []
    while 10**power <= border_hi:
        tickpos += [10**power]
        ticklabel += ["$10^{"+str(power)+"}$"]
        power += 1
    ax.set_xticks(tickpos)
    ax.set_xticklabels(ticklabel)
    ax.set_yticks(tickpos)
    ax.set_yticklabels(ticklabel)
    #plt.minorticks_off()
    print(f"potticks {tickpos} {ticklabel}")
elif tickslist:
    ax.set_xticklabels(tickslist)
    ax.set_xticks([float(x) for x in tickslist])
    ax.set_yticklabels(tickslist)
    ax.set_yticks([float(x) for x in tickslist])
    plt.minorticks_off()

plt.grid(color='#dddddd', linestyle='-', linewidth=1)

plt.plot([border_lo, lim], [border_lo, lim], 'black', alpha=0.3, linestyle="--")
#plt.plot([border_lo, border_hi], [10*border_lo, 10*border_hi], 'gray', alpha=0.3, linestyle="--", label="y=10x")
if y2:
    plt.plot([1/y2*border_lo, 1/y2*border_hi], [border_lo, border_hi], 'black', alpha=0.3, linestyle="-.", label="y="+str(y2)+"x")

plt.plot([border_lo, lim], [lim, lim], 'black', alpha=1)
plt.plot([lim, lim], [border_lo, lim], 'black', alpha=1)
plt.fill_between([border_lo, lim], [lim, lim], [border_hi, border_hi], alpha=0.3, color='gray', zorder=0) #color='blue', label=str(timeouts_y) + " timeouts of LEFT")
plt.fill_between([lim, border_hi], [border_lo, border_lo], [lim, lim], alpha=0.3, color='gray', zorder=0) #, label=str(timeouts_x) + " timeouts of BOTTOM")

timeouts_x = 0
timeouts_y = 0
label_idx = 0

def rgb_to_hex(r, g, b):
    return '#{:02x}{:02x}{:02x}'.format(r, g, b)

X = []
Y = []
keys = list(set([i for i in values[0]] + [i for i in values[1]]))

max_first_stats = max([x[stats[0]] for x in statistics[0].values()] + [x[stats[0]] for x in statistics[1].values()])
max_second_stat = max([x[stats[1]] for x in statistics[0].values()] + [x[stats[1]] for x in statistics[1].values()])

for i in keys:
    if i not in values[0] and i not in values[1]:
        continue
    
    elif i not in values[0]:
        Y += [values[1][i]]
        X += [out]
        timeouts_x += 1
        print(str(i) + " : X timeout , Y " + str(values[1][i]))
    elif i not in values[1]:
        X += [values[0][i]]
        Y += [out]
        timeouts_y += 1
        print(str(i) + " : X " + str(values[0][i]) + ", Y timeout")
    else:
        X += [values[0][i]]
        Y += [values[1][i]]
        print(str(i) + " : X " + str(values[0][i]) + ", Y " + str(values[1][i]))

    if i in statistics[0]:
        color = (rgb_to_hex(0, int(int(statistics[0][i][stats[0]])/ max_first_stats * 255 ), int(int(statistics[0][i][stats[1]])/ max_second_stat *255 )))
    else:
        color = (rgb_to_hex(0, int(int(statistics[1][i][stats[0]]) / max_first_stats * 255), int(int(statistics[1][i][stats[1]]) / max_second_stat *255)))
    plt.plot(X[-1], Y[-1], marker='.', alpha=1, markersize=msize, markeredgecolor=color, color=color)
    
label_idx += 1


if heading:
    plt.title(heading)
if xlabel:
    plt.xlabel(xlabel)
else:    
    plt.xlabel(labels[0])
if ylabel:
    plt.ylabel(ylabel)
else:    
    plt.ylabel(labels[1])

#if not outfile_legend:    
#    if legendright:
#        plt.legend(bbox_to_anchor=(1.05+legend_offset_x, 0.5+legend_offset_y), loc='center left', edgecolor="black", ncol=legend_columns, labelspacing=legend_spacing, columnspacing=legend_spacing*2, handlelength=legend_spacing*2)
#    elif legendbottom:
#        plt.legend(bbox_to_anchor=(0.5+legend_offset_x, -0.27+legend_offset_y), loc='lower center', edgecolor="black", ncol=legend_columns, labelspacing=legend_spacing, columnspacing=legend_spacing*2, handlelength=legend_spacing*2)
#    else:
#        plt.legend(ncol=legend_columns, labelspacing=legend_spacing, columnspacing=legend_spacing*2, handlelength=legend_spacing*2)

plt.tight_layout()
if outfile:
    plt.savefig(outfile)
else:
    plt.show()



def flip_legend_labels(labels, plots, ncols):

    newlabels = [l for l in labels]
    newplots = [b for b in plots]
    r = 0
    c = 0

    for i in range(len(labels)):
        j = ncols * r + c
        newlabels[i] = labels[j]
        newplots[i] = plots[j]
        if ncols * (r+1) + c >= len(labels):
            r = 0
            c += 1
        else:
            r += 1
    
    return (newlabels, newplots)

if outfile_legend:
    
    plots = []
    domains = []
    for domain in domain_to_style_map:
        style = domain_to_style_map[domain]
        domains += [domain]
        plots += [matplotlib.lines.Line2D([], [], c=style[0], marker=style[1], lw=0, fillstyle='none', label=domain)]
    figlegend = plt.figure()
    ncols = 4
    (newlabels, newplots) = flip_legend_labels(domains, plots, ncols)
    print(newlabels)
    labelspacing = 0.2
    #if labelspacing is not None:
    #    figlegend.legend(newplots, newlabels, 'center', ncol=ncols, edgecolor='#000000', labelspacing=labelspacing)
    #else:
    #    figlegend.legend(newplots, newlabels, 'center', ncol=ncols, edgecolor='#000000')
    #figlegend.set_edgecolor('b')
    #figlegend.tight_layout()
    #figlegend.savefig(outfile_legend, bbox_inches='tight')