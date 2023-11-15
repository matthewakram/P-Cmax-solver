import sys
from pysat.pb import *
import fileinput

position_vars = []

first_line = input()
first_line = first_line.split()
num_jobs = int(first_line[0])
num_procs = int(first_line[1])
next_free_var = int(first_line[2])
next_free_var -= 1
makespan = int(first_line[3])
line = input()
weights = [int(x) for x in line.strip().split()]
for i in range(0,num_jobs):
    line = input()
    position_vars.append([int(x) for x in line.strip().split()])
    
# now I have all the position vars, all I need to do is to get the clauses
clauses = []
for proc in range(num_procs):
    vars = [position_vars[i][proc] for i in range(num_jobs)]
    filtered_weights = [weights[job] for job in range(num_jobs) if vars[job] != 0]
    vars = [x for x in vars if x != 0]
    cnf = PBEnc.atmost(lits=vars, weights=filtered_weights, bound=makespan, top_id=next_free_var, encoding=EncType.binmerge)
    next_free_var = cnf.nv
    clauses += cnf.clauses
    
out = "\n".join([" ".join([str(y) for y in x]) for x in clauses])
sys.stdout.write(out)
