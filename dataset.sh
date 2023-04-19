#!/usr/bin/env bash

# cut -f1,3 title.basics.tsv > title.tsv
# grep -E 'EN|PL' title.akas.tsv | cut -f1,3 > title.tsv
awk -F'\t' 'BEGIN {OFS = FS} { if ($2 == "movie" || $2 == "tvMovie" || $2 == "tvSeries" || $2 == "tvMiniSeries") print $1, $3 }' title.basics.tsv > title.tsv


