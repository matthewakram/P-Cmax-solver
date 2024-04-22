from typing import Set



files = ["./bench/results/complete_cdsm_base.txt", 
                  "./bench/results/complete_cdsm_last_size.txt",
                  "./bench/results/complete_cdsm_inter.txt",
                  "./bench/results/complete_cdsm_fur.txt",
                  "./bench/results/complete_cdsm_irrelevance.txt",
                  "./bench/results/complete_cdsm.txt",
                  "./bench/results/complete_ilp_original.txt",
                  "./bench/results/complete_ilp.txt",
                  "./bench/results/complete_sat.txt",
                  "./bench/results/complete_hj.txt"
                  ]

correctly_bounded: Set[str] = set()

for file in files:
    correctly_bounded_for_file: Set[str] = set()
    with open(file) as f:
        lines = f.readlines()
        lines = [x.split()[:2] for x in lines]
        for line in lines:
            if line[1] == "0":
                correctly_bounded.add(line[0])
                correctly_bounded_for_file.add(line[0])
    num_class = len([x for x in correctly_bounded_for_file if "/p_cmax-E" in x or "/p_cmax-BIG" in x])
    num_franca = len([x for x in correctly_bounded_for_file if "/U_" in x or "/NU_" in x])
    num_lawrinenko = len([x for x in correctly_bounded_for_file if "p_cmax-class" in x])
    num_sc2022 = len([x for x in correctly_bounded_for_file if "-isc22-" in x])
    anni = len([x for x in correctly_bounded_for_file if "-anni-" in x])
    raxml = len([x for x in correctly_bounded_for_file if "raxml" in x or "ft-mapreduce" in x])
    sichash = len([x for x in correctly_bounded_for_file if "-sichash-" in x]) 
    taxi = len([x for x in correctly_bounded_for_file if "_KaRRi-" in x])
    graph = len([x for x in correctly_bounded_for_file if "-graph-" in x])
    cnf = len([x for x in correctly_bounded_for_file if ".cnf." in x])
    planted = len([x for x in correctly_bounded_for_file if "-planted-" in x])

    print("===============================" + file + "=============")
    print("num class that were solved by bounding " + str(num_class))
    print("num franca that were solved by bounding " + str(num_franca))
    print("num lawrinenko that were solved by bounding " + str(num_lawrinenko))
    print("num satpar that were solved by bounding " + str(anni))
    print("num sc2022 that were solved by bounding " + str(num_sc2022))
    print("num raxml that were solved by bounding " + str(raxml))
    print("num sichash that were solved by bounding " + str(sichash))
    print("num taxi that were solved by bounding " + str(taxi))
    print("num graph that were solved by bounding " + str(graph))
    print("num cnf that were solved by bounding " + str(cnf))
    print("num planted that were solved by bounding " + str(planted))

num_class = len([x for x in correctly_bounded if "/p_cmax-E" in x or "/p_cmax-BIG" in x])
num_franca = len([x for x in correctly_bounded if "/U_" in x or "/NU_" in x])
num_lawrinenko = len([x for x in correctly_bounded if "p_cmax-class" in x])
num_sc2022 = len([x for x in correctly_bounded if "-isc22-" in x])
anni = len([x for x in correctly_bounded if "-anni-" in x])
raxml = len([x for x in correctly_bounded if "raxml" in x or "ft-mapreduce" in x])
sichash = len([x for x in correctly_bounded if "-sichash-" in x]) 
taxi = len([x for x in correctly_bounded if "_KaRRi-" in x])
graph = len([x for x in correctly_bounded if "-graph-" in x])
cnf = len([x for x in correctly_bounded if ".cnf." in x])
planted = len([x for x in correctly_bounded if "-planted-" in x])


print("=============================== IN TOTAL =============")
print("num class that were solved by bounding " + str(num_class))
print("num franca that were solved by bounding " + str(num_franca))
print("num lawrinenko that were solved by bounding " + str(num_lawrinenko))
print("num satpar that were solved by bounding " + str(anni))
print("num sc2022 that were solved by bounding " + str(num_sc2022))
print("num raxml that were solved by bounding " + str(raxml))
print("num sichash that were solved by bounding " + str(sichash))
print("num taxi that were solved by bounding " + str(taxi))
print("num graph that were solved by bounding " + str(graph))
print("num cnf that were solved by bounding " + str(cnf))
print("num planted that were solved by bounding " + str(planted))

for file in files:
    set_for_file = set()
    print("===============================" + file + "=============")
    with open(file + ".filtered", 'w') as fw:
        with open(file) as f:
            lines = f.readlines()
            for line in lines:
                splited = line.split()
                if float(splited[1]) > 500.0:
                    continue
                if not splited[0] in correctly_bounded:
                    set_for_file.add(splited[0])
                    fw.write(line)
        for instance in correctly_bounded:
            fw.write(" ".join([instance, "0", "0.0", "0.0", "0.0","0.0","0.0","0.0","0.0","0.0","0.0","0.0",] ) + "\n")
        num_class = len([x for x in set_for_file if "/p_cmax-E" in x or "/p_cmax-BIG" in x])
        num_franca = len([x for x in set_for_file if "/U_" in x or "/NU_" in x])
        num_lawrinenko = len([x for x in set_for_file if "p_cmax-class" in x])
        num_sc2022 = len([x for x in set_for_file if "-isc22-" in x])
        anni = len([x for x in set_for_file if "-anni-" in x])
        raxml = len([x for x in set_for_file if "raxml" in x or "ft-mapreduce" in x])
        sichash = len([x for x in set_for_file if "-sichash-" in x]) 
        taxi = len([x for x in set_for_file if "_KaRRi-" in x])
        graph = len([x for x in set_for_file if "-graph-" in x])
        cnf = len([x for x in set_for_file if ".cnf." in x])
        planted = len([x for x in set_for_file if "-planted-" in x])



        print("num class that were solved by technique " + str(num_class))
        print("num franca that were solved by technique " + str(num_franca))
        print("num lawrinenko that were solved by technique " + str(num_lawrinenko))
        print("num satpar that were solved by technique " + str(anni))
        print("num sc2022 that were solved by technique " + str(num_sc2022))
        print("num raxml that were solved by technique " + str(raxml))
        print("num sichash that were solved by technique " + str(sichash))
        print("num taxi that were solved by technique " + str(taxi))
        print("num graph that were solved by technique " + str(graph))
        print("num cnf that were solved by technique " + str(cnf))
        print("num planted that were solved by technique " + str(planted))
        print("num total "+ str(num_class + num_franca + num_lawrinenko + anni + num_sc2022 + raxml + sichash + taxi + graph + cnf + planted))

