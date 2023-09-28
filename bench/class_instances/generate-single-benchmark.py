
import sys
import random

if len(sys.argv) < 4:
    print(f"Usage: {sys.argv[0]} n-jobs n-machines max-job-size [seed]")
    exit(0)

n = int(sys.argv[1])
m = int(sys.argv[2])
maxjobsize = int(sys.argv[3])
if len(sys.argv) >= 5:
    random.seed(int(sys.argv[4]))

print(f"p p_cmax {n} {m}")
jobdescription = ""
for i in range(n):
    jobsize = random.randrange(1, maxjobsize+1)
    assert(jobsize >= 1 and jobsize <= maxjobsize)
    jobdescription += str(jobsize) + " "
print(jobdescription + "0")
