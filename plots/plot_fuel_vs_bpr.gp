# CSTNSystems/minotaur - TSFC proxy vs Bypass Ratio
# Columns: case(1),bpr(2),opr(3),mach(4),alt_km(5),status(6),converged(7),iter(8),
#          mass_resid(9),energy_resid(10),t4(11),tsfc_proxy(12),thrust_proxy(13)

set terminal pngcairo size 1200,800 enhanced font 'Fira Code,11'
set output "results/fig_fuel_vs_bpr.png"
set datafile separator ","

set style line 1 lc rgb '#0060ad' pt 7 ps 0.8 lt 1 lw 1.5
set style line 2 lc rgb '#dd181f' pt 5 ps 0.8 lt 1 lw 1.5

set grid xtics ytics ls -1 lc rgb '#e0e0e0'
set border 3 lw 1.5
set tics nomirror

set title "CSTNSystems/minotaur - TSFC proxy vs Bypass Ratio (sweep)" font ',14'
set xlabel "Bypass Ratio (BPR)" font ',12'
set ylabel "TSFC proxy" font ',12'

set key right top box opaque

# Only plot converged points (status == 0)
plot "results/out_sweep.csv" using 2:($6==0 ? $12 : 1/0) with points ls 1 title "TSFC (converged)", \
     "" using 2:($6!=0 ? $12 : 1/0) with points ls 2 title "TSFC (failed)"
