# Feasability Tests

We had tried using feasability checking to avoid processing certain makespans. 
This is because we know that checking certain makespans is useless, if they e.g. cannot be optimal makespans. 
Unfortunately, the only feasability test I can think of is the subset sum test
But it is inefective at making the encoding easier. This is because, if between makespan and makespan' there is no
reachable makespan (by subset sum) then the encoding does not change. 
We need other feasability tests.