import multiprocessing
import subprocess

files = ["./bench/cumulative/p_cmax-class2-n72-m32-minsize20-maxsize100-seed2959.txt",
"./bench/cumulative/p_cmax-n21-m5-sichash-100Mkeys-sample21-easy.txt",
"./bench/cumulative/p_cmax-class7-n36-m16-mu144-sigma36-seed31465.txt",
"./bench/cumulative/p_cmax-class6-n22-m8-minsize22-maxsize88-seed27548.txt",
"./bench/cumulative/p_cmax-class6-n100-m50-minsize100-maxsize400-seed16756.txt",
"./bench/cumulative/p_cmax-E2-n30-m10-minsize100-maxsize800-seed13680.txt",
"./bench/cumulative/p_cmax-class2-n36-m16-minsize20-maxsize100-seed5432.txt",
"./bench/cumulative/p_cmax-E3-n32-m10-minsize100-maxsize200-seed25378.txt",
"./bench/cumulative/p_cmax-class3-n90-m40-minsize50-maxsize100-seed26267.txt",
"./bench/cumulative/p_cmax-class6-n54-m24-minsize54-maxsize216-seed188.txt",
"./bench/cumulative/p_cmax-class5-n36-m12-mu100-sigma50-seed6940.txt",
"./bench/cumulative/p_cmax-class7-n160-m80-mu640-sigma160-seed20671.txt",
"./bench/cumulative/p_cmax-class7-n36-m16-mu144-sigma36-seed789.txt",
"./bench/cumulative/p_cmax-class7-n36-m16-mu144-sigma36-seed5641.txt",
"./bench/cumulative/p_cmax-class2-n54-m24-minsize20-maxsize100-seed13118.txt",
"./bench/cumulative/p_cmax-class7-n22-m8-mu88-sigma22-seed10127.txt",
"./bench/cumulative/p_cmax-class5-n180-m80-mu100-sigma50-seed8831.txt",
"./bench/cumulative/p_cmax-class5-n108-m48-mu100-sigma50-seed2306.txt",
"./bench/cumulative/p_cmax-E3-n31-m10-minsize1-maxsize100-seed3774.txt",
"./bench/cumulative/p_cmax-class3-n40-m16-minsize50-maxsize100-seed19874.txt",
"./bench/cumulative/p_cmax-class7-n200-m100-mu800-sigma200-seed5023.txt",
"./bench/cumulative/p_cmax-E3-n26-m8-minsize100-maxsize200-seed12836.txt",
"./bench/cumulative/p_cmax-class3-n108-m48-minsize50-maxsize100-seed31445.txt",
"./bench/cumulative/p_cmax-E3-n26-m8-minsize100-maxsize200-seed14984.txt",
"./bench/cumulative/p_cmax-E3-n25-m8-minsize100-maxsize200-seed9764.txt",
"./bench/cumulative/p_cmax-class7-n180-m90-mu720-sigma180-seed30557.txt",
"./bench/cumulative/p_cmax-E3-n26-m8-minsize100-maxsize200-seed20291.txt",
"./bench/cumulative/p_cmax-class5-n144-m64-mu100-sigma50-seed19808.txt",
"./bench/cumulative/p_cmax-class2-n54-m24-minsize20-maxsize100-seed23159.txt",
"./bench/cumulative/p_cmax-n21-m4-sichash-100Mkeys-sample21-easy.txt",
"./bench/cumulative/p_cmax-class4-n54-m24-mu100-sigma20-seed7806.txt",
"./bench/cumulative/p_cmax-class6-n80-m40-minsize80-maxsize320-seed13924.txt",
"./bench/cumulative/p_cmax-class5-n36-m16-mu100-sigma50-seed16801.txt",
"./bench/cumulative/p_cmax-class2-n54-m24-minsize20-maxsize100-seed28902.txt",
"./bench/cumulative/p_cmax-class6-n160-m80-minsize160-maxsize640-seed15090.txt",
"./bench/cumulative/p_cmax-n50-m20-planted-exact-U3000-perturb0.05.txt",
"./bench/cumulative/p_cmax-E3-n25-m8-minsize100-maxsize200-seed252.txt",
"./bench/cumulative/p_cmax-class3-n90-m40-minsize50-maxsize100-seed258.txt",
"./bench/cumulative/p_cmax-class1-n200-m80-minsize1-maxsize100-seed25387.txt",
"./bench/cumulative/p_cmax-class6-n160-m80-minsize160-maxsize640-seed31197.txt",
"./bench/cumulative/p_cmax-class2-n54-m24-minsize20-maxsize100-seed8824.txt",
"./bench/cumulative/p_cmax-class7-n160-m80-mu640-sigma160-seed6415.txt",
"./bench/cumulative/p_cmax-class1-n44-m16-minsize1-maxsize100-seed31821.txt",
"./bench/cumulative/p_cmax-E3-n31-m10-minsize1-maxsize100-seed14052.txt",
"./bench/cumulative/p_cmax-E2-n30-m10-minsize100-maxsize800-seed30892.txt",
"./bench/cumulative/p_cmax-E3-n26-m8-minsize100-maxsize200-seed3708.txt",
"./bench/cumulative/p_cmax-class1-n162-m72-minsize1-maxsize100-seed18941.txt",
"./bench/cumulative/p_cmax-class7-n140-m70-mu560-sigma140-seed10070.txt",
"./bench/cumulative/p_cmax-class7-n180-m90-mu720-sigma180-seed30369.txt",
"./bench/cumulative/p_cmax-n125-m50-planted-exact-U1000-perturb0.01.txt",
"./bench/cumulative/p_cmax-E3-n26-m8-minsize100-maxsize200-seed28169.txt",
"./bench/cumulative/p_cmax-E3-n31-m10-minsize1-maxsize100-seed18670.txt",
"./bench/cumulative/p_cmax-n125-m50-planted-exact-U1000-perturb0.1.txt",
"./bench/cumulative/p_cmax-class7-n200-m100-mu800-sigma200-seed9179.txt",
"./bench/cumulative/p_cmax-n50-m20-planted-exact-U1000-perturb0.01.txt",
"./bench/cumulative/p_cmax-class1-n40-m16-minsize1-maxsize100-seed14476.txt",
"./bench/cumulative/p_cmax-class2-n54-m24-minsize20-maxsize100-seed30841.txt",
"./bench/cumulative/p_cmax-class2-n36-m12-minsize20-maxsize100-seed19824.txt",
"./bench/cumulative/p_cmax-class2-n22-m8-minsize20-maxsize100-seed25434.txt",
"./bench/cumulative/p_cmax-class6-n36-m16-minsize36-maxsize144-seed2265.txt",
"./bench/cumulative/p_cmax-class3-n108-m48-minsize50-maxsize100-seed1689.txt",
"./bench/cumulative/p_cmax-E3-n26-m8-minsize100-maxsize200-seed20981.txt",
"./bench/cumulative/p_cmax-class3-n22-m8-minsize50-maxsize100-seed10640.txt",
"./bench/cumulative/p_cmax-class1-n160-m64-minsize1-maxsize100-seed20270.txt",
"./bench/cumulative/p_cmax-class5-n54-m24-mu100-sigma50-seed29098.txt",
"./bench/cumulative/p_cmax-class3-n72-m32-minsize50-maxsize100-seed12579.txt",
"./bench/cumulative/p_cmax-class7-n60-m24-mu240-sigma60-seed6036.txt",
"./bench/cumulative/p_cmax-E2-n30-m10-minsize100-maxsize800-seed4347.txt",
"./bench/cumulative/p_cmax-class7-n40-m16-mu160-sigma40-seed13476.txt",
"./bench/cumulative/p_cmax-E3-n25-m8-minsize100-maxsize200-seed2808.txt",
"./bench/cumulative/p_cmax-class7-n22-m8-mu88-sigma22-seed24776.txt",
"./bench/cumulative/p_cmax-n40-m10-planted-exact-U3000-perturb0.05.txt",
"./bench/cumulative/p_cmax-E3-n26-m8-minsize100-maxsize200-seed24757.txt",
"./bench/cumulative/p_cmax-class3-n44-m16-minsize50-maxsize100-seed9690.txt",
"./bench/cumulative/p_cmax-class5-n22-m8-mu100-sigma50-seed14483.txt",
"./bench/cumulative/p_cmax-E3-n25-m8-minsize100-maxsize200-seed260.txt",
"./bench/cumulative/p_cmax-class6-n200-m100-minsize200-maxsize800-seed24554.txt",
"./bench/cumulative/p_cmax-class1-n180-m72-minsize1-maxsize100-seed20280.txt",
"./bench/cumulative/p_cmax-class7-n36-m12-mu144-sigma36-seed6063.txt",
"./bench/cumulative/p_cmax-E3-n32-m10-minsize1-maxsize100-seed21959.txt",
"./bench/cumulative/p_cmax-n250-m100-planted-exact-U1000-perturb0.01.txt",
"./bench/cumulative/p_cmax-class1-n180-m72-minsize1-maxsize100-seed31561.txt",
"./bench/cumulative/p_cmax-class3-n90-m40-minsize50-maxsize100-seed903.txt",
"./bench/cumulative/p_cmax-E3-n26-m8-minsize100-maxsize200-seed2070.txt",
"./bench/cumulative/p_cmax-class7-n40-m16-mu160-sigma40-seed26312.txt",
"./bench/cumulative/p_cmax-class7-n140-m70-mu560-sigma140-seed1321.txt",
"./bench/cumulative/p_cmax-E3-n31-m10-minsize100-maxsize200-seed28969.txt",
"./bench/cumulative/p_cmax-class5-n36-m12-mu100-sigma50-seed9760.txt",
"./bench/cumulative/p_cmax-class3-n40-m16-minsize50-maxsize100-seed30374.txt",
"./bench/cumulative/NU_2_0050_10_3.txt",
"./bench/cumulative/p_cmax-class7-n200-m100-mu800-sigma200-seed26440.txt",
"./bench/cumulative/p_cmax-E2-n30-m10-minsize100-maxsize800-seed23353.txt",
"./bench/cumulative/p_cmax-class3-n44-m16-minsize50-maxsize100-seed17471.txt",
"./bench/cumulative/p_cmax-n50-m20-planted-exact-U1000-perturb0.1.txt"]



def func(x):
    subprocess.run(["cargo", "run", "-r", x, "-intercomp", "-prec", "-t", "500"], stdout=subprocess.DEVNULL)

if __name__ == "__main__":
    pool = multiprocessing.Pool(100)
    print(files)
    pool.map(func, files)