#!/usr/bin/env python
# coding: utf-8
"""
Plotting a stream of data
Usage: python script.py filename
"""

from glob import glob
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import os
import sys

def main(data, stream_plot=False, threshold=.90):
    till = None
    block = 128
    result_ones = np.array([0]*block).astype(float)
    candidates = []

    plt.ion()
    for i,val in enumerate(data[slice(None,till,None)]):
        for j in range(8):
            num = (val & (1 << j)) > 0
            minus = result_ones[0]
            result_ones = np.roll(result_ones, -1)
            result_ones = result_ones - minus
            result_ones[-1] = result_ones[-2] + float(num)
            if i*8+j > block:
                correlations = [abs(np.corrcoef(c, result_ones))[0,1] for c in candidates]
                if all([cc<threshold for cc in correlations]):
                    candidates.append(result_ones/block)
                    pd.DataFrame(candidates).T.to_csv('/tmp/tmp.corrs.csv')
            elif i*8+j == block:
                candidates.append(result_ones/block)
            if stream_plot:
                print(result_ones/block)
                plt.clf()
                plt.plot(range(i*8+j-block,i*8+j),result_ones/block)
                plt.ylim(0,1)
                plt.title(str(i*8+j))
                plt.draw()
                plt.pause(0.001)
    return candidates

if __name__ == "__main__":
    filename = sys.argv[1]
    vdata = np.fromfile(filename, dtype='uint8')

    stream_plot = False
    threshold = .9

    c = main(vdata, stream_plot=stream_plot, threshold=threshold)
    bname = os.path.basename(filename)
    if stream_plot:
        plt.close()

    for can in c:
        plt.plot(can)
    plt.title(bname)
    pd.DataFrame(c).T.to_csv(bname+'.corrs.csv')
    plt.savefig(bname+'.corrs.svg', format='svg')
