# Run one kappa-conversion job single-threaded and report peak RSS (ru_maxrss of children) and wall time.
import resource, subprocess, sys, time, os, json
env = dict(os.environ, RAYON_NUM_THREADS="1", OMP_NUM_THREADS="1")
t0 = time.time()
p = subprocess.run(["nice", "-n", "10", "timeout", "600"] + sys.argv[2:], env=env, capture_output=True, text=True)
wall = time.time() - t0
rss_mb = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss / 1024
row = {"label": sys.argv[1], "exit": p.returncode, "peak_rss_mb": round(rss_mb, 1), "wall_s": round(wall, 1),
       "stderr_tail": p.stderr.strip().splitlines()[-3:]}
print(json.dumps(row)); open("mem_runs.jsonl", "a").write(json.dumps(row) + "\n")
