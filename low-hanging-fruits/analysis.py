#!/usr/bin/env python
# coding: utf-8
"""
Analysis of the MQ encoder results
"""
import numpy as np
import pandas as pd
from matplotlib import pyplot as plt
import seaborn as sns
from glob import glob
import os
from itertools import chain


def main():
    df = get_datafile_sizes()
    print(df)


def get_datafile_sizes():
    basenames = glob("./*.residual")
    uncompressed = dict(
        residual = sorted([x for x in basenames]),
        nolzc = sorted([x+'.nlzc' for x in basenames]),
        bplanes = sorted([x+'.bplanes' for x in basenames]),
    )
    compressed = dict(
        residual = sorted([x+'.mq' for x in basenames]),
        nolzc = sorted([x+'.nlzc.mq' for x in basenames]),
        bplanes = sorted([x+'.bplanes.mq' for x in basenames]),
    )
    merged = chain.from_iterable([x for x in uncompressed.values()]+[x for x in compressed.values()])
    merged = [x for x in merged]

    # Next cell
    index = [x[2:-9] for x in basenames]
    df = pd.DataFrame([], index=index)
    df['residual'] = [os.path.getsize(x+'.residual') for x in df.index]
    df['nlzc'] = [os.path.getsize(x+'.residual.nlzc') for x in df.index]
    df['bplanes'] = [os.path.getsize(x+'.residual.bplanes') for x in df.index]
    df['residual.mq'] = [os.path.getsize(x+'.residual.mq') for x in df.index]
    df['nlzc.mq'] = [os.path.getsize(x+'.residual.nlzc.mq') for x in df.index]
    df['bplanes.mq'] = [os.path.getsize(x+'.residual.bplanes.mq') for x in df.index]

    # CR
    df['residual.cr'] = df['residual.mq']/df['residual']
    df['nlzc.cr'] = df['nlzc.mq']/df['nlzc']
    df['bplanes.cr'] = df['bplanes.mq']/df['bplanes']
    df.sort_index(inplace=True)

    # Cleanup
    nlzc_too_small = df.loc[df['nlzc']<1000000].index
    for x in nlzc_too_small:
        df.drop(x, axis=0, inplace=True)

    return df
