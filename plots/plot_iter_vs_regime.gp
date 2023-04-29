# CSTNSystems/minotaur - Iterations vs OPR (convergence behavior)
# Columns: case(1),bpr(2),opr(3),mach(4),alt_km(5),status(6),converged(7),iter(8),
#          mass_resid(9),energy_resid(10),t4(11),tsfc_proxy(12),thrust_proxy(13)

set terminal pngcairo size 1200,800 enhanced font 'Fira Code,11'
set output "results/fig_iter_vs_regime.png"
set datafile separator ","

set style line 1 lc rgb '#0060ad' pt 7 ps 0.8 lt 1 lw 1.5
set style line 2 lc rgb '#dd181f' pt 5 ps 0.8 lt 1 lw 1.5
set style line 3 lc rgb '#00aa00' pt 9 ps 0.8 lt 1 lw 1.5

set grid xtics ytics ls -1 lc rgb '#e0e0e0'
set border 3 lw 1.5
set tics nomirror

set title "CSTNSystems/minotaur - Solver iterations vs OPR (sweep)" font ',14'
set xlabel "Overall Pressure Ratio (OPR)" font ',12'
set ylabel "Iteration count" font ',12'

set key right top box opaque

# Color by convergence status
plot "results/out_sweep.csv" using 3:($6==0 ? $8 : 1/0) with points ls 1 title "converged", \
     "" using 3:($6==1 ? $8 : 1/0) with points ls 2 title "maxiter", \
     "" using 3:($6>=2 ? $8 : 1/0) with points ls 3 title "failed"
