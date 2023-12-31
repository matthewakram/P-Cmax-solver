use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use rand::{rngs::ThreadRng, Rng};

use crate::{
    common, encoding::cplex_model_encoder::CPLEXModelEncoder, problem_instance::solution::Solution,
};

#[derive(Clone)]
pub struct PinarSeyda {
    encoding: String,
    model_file_string: String,
}

impl PinarSeyda {
    pub fn new() -> PinarSeyda {
        let mut rng: ThreadRng = ThreadRng::default();
        let mut i: usize = rng.gen();
        let mut file_name = format!("./ParalelMachineScheduling_CP{}.mod", i);
        while Path::new(&file_name).is_file() {
            i = rng.gen();
            file_name = format!("./ParalelMachineScheduling_CP{}.mod", i);
        }

        let mut file = File::create(Path::new(&file_name)).expect("unable to create model file");
        file.write(CODE.as_bytes())
            .expect("could not write model to file");

        return PinarSeyda {
            encoding: String::new(),
            model_file_string: file_name,
        };
    }
}

impl Drop for PinarSeyda {
    fn drop(&mut self) {
        fs::remove_file(&self.model_file_string).expect("could not delete file to drop object");
    }
}

impl CPLEXModelEncoder for PinarSeyda {
    fn encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        lower_bound: usize,
        makespan: usize,
        _timeout: &crate::common::timeout::Timeout,
    ) -> bool {
        let mut zeroes_jobs: String = "0".to_owned();
        for _ in 0..partial_solution.instance.num_jobs {
            zeroes_jobs += " 0";
        }

        let mut zeroes_procs: String = "0".to_owned();
        for _ in 1..partial_solution.instance.num_processors {
            zeroes_procs += " 0";
        }
        let mut job_sizes: String = format!("[{}]", zeroes_procs);

        for i in 0..partial_solution.instance.num_jobs {
            let mut size: String = format!("{}", partial_solution.instance.job_sizes[i]);
            for _ in 1..partial_solution.instance.num_processors {
                size += &format!(" {}", partial_solution.instance.job_sizes[i]);
            }
            job_sizes += &format!(", [{}]", size);
        }

        let mut precedences: String = String::new();
        for i in 0..partial_solution.instance.num_jobs + 1 {
            let comma = if i == partial_solution.instance.num_jobs {
                ""
            } else {
                ","
            };
            precedences += &format!("<{}, {{}}>{}", i, comma);
        }

        let mut eligible = String::new();
        for i in 0..partial_solution.instance.num_jobs + 1 {
            let comma = if i == partial_solution.instance.num_jobs {
                ""
            } else {
                ","
            };
            let mut eligible_i = String::from("1");
            for j in 1..partial_solution.instance.num_processors {
                if i == 0 || partial_solution.possible_allocations[i - 1].contains(&j) {
                    eligible_i += &format!(" {}", j + 1);
                } else {
                    eligible_i += &" 0";
                }
            }
            eligible += &format!("[{}]{}", eligible_i, comma);
        }
        let mut set_up_times = format!("[{}]", zeroes_jobs);
        for _ in 0..partial_solution.instance.num_jobs {
            set_up_times += &format!(",\n[{}]", zeroes_jobs);
        }
        let mut setup_times_for_all_procs = format!("[{}]", set_up_times);
        for _ in 1..partial_solution.instance.num_processors {
            setup_times_for_all_procs += &format!(",\n[{}]", set_up_times);
        }

        self.encoding = format!("nbJobs = {};\nnbMchns = {};\nLB={};\nUB={};\nnbResources = 0;\ncapResource = [];\nRelease = [{}];\ndemandR = [];\nprocessTime = [{}];\nPrecedences = {{ {} }};\nmEligible=[{}];\nsetup = [{}];", partial_solution.instance.num_jobs+1, partial_solution.instance.num_processors, lower_bound, makespan,  zeroes_jobs, job_sizes, precedences, eligible, setup_times_for_all_procs);
        return true;
    }

    fn get_encoding(&self) -> String {
        return self.encoding.clone();
    }

    fn get_mod_file_path(&self) -> String {
        return self.model_file_string.clone();
    }

    fn decode(
        &self,
        instance: &crate::problem_instance::problem_instance::ProblemInstance,
        solution: String,
    ) -> crate::problem_instance::solution::Solution {
        let objective_pointer = solution.find("OBJECTIVE: ").unwrap();
        let (objective, _) = solution[objective_pointer + 11..].split_once("\n").unwrap();
        let found_makespan = objective.parse::<usize>().unwrap();
        let start_pointer = solution.find("X = [[<").unwrap();
        let end_pointer = solution.find("Mch = [{").unwrap();
        let start_pointer = start_pointer + 8;
        let end_pointer = end_pointer - 2;
        let assignments = solution[start_pointer..end_pointer].to_string();
        let assignments = assignments.as_bytes();

        let num_procs = instance.num_processors;
        let mut job_assignments: Vec<usize> = vec![];
        let mut proc_num = 0;
        let mut past_first_row = false;
        for i in 0..assignments.len() {
            let char = assignments[i];

            if char == b'<' && past_first_row {
                if assignments[i + 1] == b'1' {
                    job_assignments.push(proc_num);
                }

                proc_num += 1;
                proc_num %= num_procs;
            }
            if char == b']' {
                past_first_row = true;
            }
        }

        assert_eq!(job_assignments.len(), instance.num_jobs);

        let calculated_makespan = common::common::calc_makespan(instance, &job_assignments);

        assert_eq!(calculated_makespan, found_makespan);

        return Solution {
            makespan: calculated_makespan,
            assignment: job_assignments,
        };
    }
}

const CODE: &str = "
using CP;
float LB=...;  //LB
int UB=...; // Upper bound
int nbJobs = ...;
int nbMchns = ...;
int nbResources = ...;
range Resources = 0..nbResources-1;
range Jobs = 0..nbJobs-1;
range Mchns = 1..nbMchns;
int Release[Jobs] = ...; //R
int capResource[Resources]= ...; //AR
int demandR[Resources][Jobs]= ...; //Res
int processTime[Jobs][Mchns] = ...; //P
int mEligible[Jobs][Mchns] = ...; //Eg

int setup[Mchns][Jobs][Jobs] = ...;
tuple triplet { int j1; int j2; int time; } 
{triplet} transitionTime[m in Mchns] = 
 { <j1,j2,setup[m][j1][j2]> | j1 in Jobs, j2 in Jobs};

tuple precedences { key int jobId; {int} pre;}
{precedences} Precedences = ...;
 
//Decision variables
dvar interval Z[j in Jobs];
dvar interval X [j in Jobs][m in Mchns] optional size processTime[j][m];
dvar sequence Mch[m in Mchns] in all(j in Jobs) X[j][m] types all(j in Jobs) j;

//cumulFunction for Constraint (10)
cumulFunction resource [r in Resources]= sum (j in Jobs)
                       pulse(Z[j], demandR[r][j]);
                       
//cumulFunction for Constraint (11) - Redundant
cumulFunction mchUsage = sum (j in Jobs) pulse(Z[j],1);


dexpr int Cmax = max (j in Jobs) endOf(Z[j]);

execute{
       cp.param.timeLimit=3600;
}

minimize Cmax;
subject to {

//Constraint (3)
forall (j in 1..nbJobs-1)
   alternative(Z[j], all(m in Mchns: mEligible[j][m]!=0) X[j][mEligible[j][m]]);


//Constraint (4)
forall(m in Mchns)
   presenceOf(X[0][m])==1;
   
//Constraint (5)	
forall(j in Jobs, m in Mchns: mEligible[j][m]==0)
 presenceOf(X[j][m])==0;

//Constraint (6)
forall (m in Mchns)
   first(Mch[m], X[0][m]);

//Constraint (7)
forall(j in Jobs, m in Mchns)
    noOverlap(Mch[m], transitionTime[m], true);
   
//Constraint (8)
forall (j in Precedences:card(j.pre)!=0, k in j.pre)
   endBeforeStart(Z[k], Z[j.jobId]);
   
//Constraint (9)
forall (j in Jobs, m in Mchns)
 startOf(X[j][m],100000)>= Release[j]+setup[m][typeOfPrev(Mch[m],X[j][m],0,0)][j];
 
//Constraint (10) - cumulFunction
forall (r in Resources)
   resource[r] <= capResource[r];


//Feasible Cuts: Constraints (11), (12), (13)

//Constraint (11) - cumulFunction - Redundant
mchUsage <= nbMchns; 
 
//Constraint (12) - Redundant
forall(j in Precedences:card(j.pre)!=0, m in Mchns)
  startOf(X[j.jobId][m],10000000)>= max(k in j.pre, l in Mchns: mEligible[k][l]!=0)endOf(X[k][mEligible[k][l]]);

//Constraint (13)
Cmax>=LB; 
//Constraint (14)
Cmax<=UB;
} 


//Branching strategy 1: X � Mch � Z  
/*execute {   
var f = cp.factory;
var phase1 = f.searchPhase(X,f.selectSmallest(f.domainSize()),f.selectSmallest(f.valueSuccessRate()));
var phase2 = f.searchPhase(Mch,f.selectSmallest(f.domainSize()),f.selectSmallest(f.valueSuccessRate()));
var phase3 = f.searchPhase(Z,f.selectSmallest(f.domainSize()),f.selectSmallest(f.valueSuccessRate()));
cp.setSearchPhases(phase1, phase2, phase3);
}*/

/*execute {
var p = cp.param;
p.RestartFailLimit = 40000;
}*/  

/**************************************************************************************************************/  

//Branching strategy 2: Z � X � Mch 
execute {   
var f = cp.factory;
var phase1 = f.searchPhase(Z,f.selectSmallest(f.domainSize()),f.selectSmallest(f.valueSuccessRate()));
var phase2 = f.searchPhase(X,f.selectSmallest(f.domainSize()),f.selectSmallest(f.valueSuccessRate()));
var phase3 = f.searchPhase(Mch,f.selectSmallest(f.domainSize()),f.selectSmallest(f.valueSuccessRate()));
cp.setSearchPhases(phase1, phase2, phase3);
}

execute {
var p = cp.param;
p.RestartFailLimit = 40000;
} 

";
