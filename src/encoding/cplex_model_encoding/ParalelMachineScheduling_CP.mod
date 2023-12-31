/*********************************************
 * OPL 12.6.1.0 Model
 * Author: P�nar Yunuso�lu
 * Creation Date: 26 May 2019 at 20:00:47
 *********************************************/
  
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
/*execute {   
var f = cp.factory;
var phase1 = f.searchPhase(Z,f.selectSmallest(f.domainSize()),f.selectSmallest(f.valueSuccessRate()));
var phase2 = f.searchPhase(X,f.selectSmallest(f.domainSize()),f.selectSmallest(f.valueSuccessRate()));
var phase3 = f.searchPhase(Mch,f.selectSmallest(f.domainSize()),f.selectSmallest(f.valueSuccessRate()));
cp.setSearchPhases(phase1, phase2, phase3);
}*/

/*execute {
var p = cp.param;
p.RestartFailLimit = 40000;
}*/  






 
