#!/usr/bin/env python
"""
Preprocessing of files : Get historgramm of all bytes in one csv file.
"""


import pandas as pd
import numpy as np
import sys


def feature_scaling(data, a=0, b=1):
    t = data - data.min()
    return a + t *(b - a) / (data.max() - data.min())

def main():
    files = sys.argv[1:]
    data = dict()

    for x in files:
        tmp = np.fromfile(x, dtype='uint8').astype('float32')
        counts, _ = np.histogram(tmp, bins=np.arange(256))
        data[x] = counts
    df = pd.DataFrame(data)
    df.index.name = 'ix'
    df.to_csv("/tmp/histogram.csv")

    # for x in files:
    #     tmp = np.fromfile(x, dtype='uint8').astype('float32')
    #     counts, _ = np.histogram(tmp, bins=np.arange(256))
    #     data[x] = feature_scaling(data)
    # df = pd.DataFrame(data)
    # df.index.name = 'ix'
    # df.to_csv("/tmp/histogram.fs.csv")

if __name__ == '__main__':
    main()
