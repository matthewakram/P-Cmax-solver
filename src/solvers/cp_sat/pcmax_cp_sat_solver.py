import sys
import fileinput
from ortools.sat.python import cp_model

position_vars = []

first_line = input()
first_line = first_line.split()
num_jobs = int(first_line[0])
num_procs = int(first_line[1])
lower_bound = int(first_line[2])
upper_bound = int(first_line[3])
time_limit = float(first_line[4])

line = input()
weights = [int(x) for x in line.strip().split()]

model = cp_model.CpModel()

position_vars_str = [[f"{job_num}_{proc_num}" for job_num in range(num_jobs)] for proc_num in range(num_procs)]

position_vars = [[model.new_bool_var(var_name) for var_name in var_name_list] for var_name_list in position_vars_str]

objective = model.new_int_var_from_domain(cp_model.Domain.from_intervals([[lower_bound, upper_bound]]), 'objective')

for job in range(num_jobs):
    model.add_exactly_one([position_vars_proc[job] for position_vars_proc in position_vars])

for proc in range(num_procs):
    model.add(sum([weights[job] * position_vars[proc][job] for job in range(num_jobs)]) <= objective)

model.minimize(objective)

solver = cp_model.CpSolver()
solver.parameters.max_time_in_seconds = time_limit
status = solver.solve(model)

if status == cp_model.OPTIMAL:
    print(f"{solver.value(objective)}")
    solution = []
    for job in range(num_jobs):
        for proc in range(num_procs):
            if solver.value(position_vars[proc][job]) == 1:
                solution.append(str(proc))
                break
    print(" ".join(solution))
elif status == cp_model.INFEASIBLE:
    print("unsatisfiable")
else:
    print('timeout')