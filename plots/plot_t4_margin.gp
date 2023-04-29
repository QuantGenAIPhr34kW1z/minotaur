# CSTNSystems/minotaur - Thermal proxy T4 vs OPR
# Columns: case(1),bpr(2),opr(3),mach(4),alt_km(5),status(6),converged(7),iter(8),
#          mass_resid(9),energy_resid(10),t4(11),tsfc_proxy(12),thrust_proxy(13)

set terminal pngcairo size 1200,800 enhanced font 'Fira Code,11'
set output "results/fig_t4_margin.png"
set datafile separator ","

set style line 1 lc rgb '#0060ad' pt 7 ps 0.8 lt 1 lw 1.5
set style line 2 lc rgb '#dd181f' pt 5 ps 0.8 lt 1 lw 1.5
set style line 3 lc rgb '#ff8c00' lt 2 lw 2 dashtype 2

set grid xtics ytics ls -1 lc rgb '#e0e0e0'
set border 3 lw 1.5
set tics nomirror

set title "CSTNSystems/minotaur - Thermal proxy T4 vs OPR (sweep)" font ',14'
set xlabel "Overall Pressure Ratio (OPR)" font ',12'
set ylabel "T4 (thermal proxy)" font ',12'

set key right bottom box opaque

# T4 max constraint line at 1400
set arrow from graph 0, first 1400 to graph 1, first 1400 nohead ls 3

# Plot converged and constraint-violated points
plot "results/out_sweep.csv" using 3:($6==0 ? $11 : 1/0) with points ls 1 title "T4 (converged)", \
     "" using 3:($6==4 ? $11 : 1/0) with points ls 2 title "T4 (constraint violated)", \
     1400 with lines ls 3 title "T4_{max} constraint"
