import shutil
import os

directories = [
        "./bench/class_instances/",
        "./bench/franca_frangioni/standardised/",
        "./bench/lawrenko/",
        "/global_data/pcmax_instances/finaler/cnf/",
        "/global_data/pcmax_instances/finaler/graph/",
        "/global_data/pcmax_instances/finaler/planted/",
        "/global_data/pcmax_instances/finaler/anni/",
        "/global_data/pcmax_instances/finaler/huebner/",
        "/global_data/pcmax_instances/finaler/lehmann/",
        "/global_data/pcmax_instances/finaler/schreiber/",
        "/global_data/pcmax_instances/finaler/laupichler/"
    ]

dest_dir = './bench/cumulative/'

for src_dir in directories:
    files = os.listdir(src_dir)
    shutil.copytree(src_dir, dest_dir, dirs_exist_ok=True)