#!/bin/bash

# This script will create files for the low-hanging-fruit analysis.
# GOAL:
# Fast creation of residual files, compressed residual files, compressed bitplane files,
# as well as MQ encoded variations of these files.
# PREANALYSIS:
# Current distributions of the MQ table. This could be giving hints about how the
# data is/was for which the MQ data was designed for. This also helps map for which
# data the probability table will/will not work.
# POSTPROCESSING:
# After this execution all files should be analysed using mqhistogram.py
# for the creation of histgrams and later the customisation of MQ encoder tables.
# DETAILS:
# The new table should be representing the different distributions occuring
# in the datasets.


# EXAMPLE:
# parallel --dry-run "./low-hanging-fruits.sh {} 47 351 901 raid6/" ::: ../pzip/data/icon.pl.*
folder=${5%/*}
r=$folder/$(basename $1).residual
cargo run --release -- compress -i $1  -o $r -s $2 $3 $4 -t f32 -p lorenz &&
cargo run --release -- mqanalysis -i $r -b $r.bplanes -n $r.nlzc &&
/home/ucyo/Developments/mqcoder/mqcoder.out -c < $r > $r.mq &&
/home/ucyo/Developments/mqcoder/mqcoder.out -c < $r.bplanes > $r.bplanes.mq &&
/home/ucyo/Developments/mqcoder/mqcoder.out -c < $r.nlzc > $r.nlzc.mq &&
exit 0
