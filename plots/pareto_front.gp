# MINOTAUR v2.9 - Pareto Front Visualization
# CSTNSystems Compact Subsonic Turbofan Numerical Systems
#
# Usage: gnuplot -e "datafile='results/pareto_front.csv'" plots/pareto_front.gp

set terminal pngcairo size 1200,900 enhanced font 'Helvetica,12'
set output 'results/pareto_front.png'

set multiplot layout 2,2 title "MINOTAUR v2.9 - NSGA-II Pareto Front Analysis" font 'Helvetica,14'

# Color palette
set palette defined (0 '#1f77b4', 1 '#ff7f0e', 2 '#2ca02c', 3 '#d62728')

# ============================================================================
# Plot 1: TSFC vs Thrust (Pareto Front)
# ============================================================================
set title "Pareto Front: TSFC vs Thrust"
set xlabel "TSFC Proxy (minimize)"
set ylabel "Thrust Proxy (maximize)"
set grid
set key top right

set datafile separator ','
plot datafile skip 1 using 7:8:($1 == 0 ? 0 : 1) with points pt 7 ps 1.5 palette notitle, \
     datafile skip 1 using 7:8 with lines lc rgb '#1f77b4' lw 1.5 notitle

# ============================================================================
# Plot 2: Design Variables Distribution (BPR vs OPR)
# ============================================================================
set title "Design Space: BPR vs OPR"
set xlabel "Bypass Ratio (BPR)"
set ylabel "Overall Pressure Ratio (OPR)"

plot datafile skip 1 using 3:4:9 with points pt 7 ps 1.5 palette title 'T4 [K]'

# ============================================================================
# Plot 3: Efficiency Parameters
# ============================================================================
set title "Efficiency Distribution"
set xlabel "Compressor Efficiency (η_c)"
set ylabel "Turbine Efficiency (η_t)"

plot datafile skip 1 using 5:6:7 with points pt 7 ps 1.5 palette title 'TSFC'

# ============================================================================
# Plot 4: Crowding Distance Distribution
# ============================================================================
set title "Crowding Distance on Pareto Front"
set xlabel "TSFC Proxy"
set ylabel "Crowding Distance"
set logscale y

plot datafile skip 1 using 7:($2 > 0 ? $2 : 0.001) with points pt 7 ps 1.5 lc rgb '#2ca02c' notitle

unset multiplot
