#!/usr/bin/env python
# coding: utf-8
"""
Transformation for ones to actual probability distributions: steadily falling or rising and all below .5
"""
import numpy as np
import pandas as pd
from matplotlib import pyplot as plt
import seaborn as sns
from itertools import chain
from functools import namedtuple

def main(filename):
    df = pd.read_csv(filename, skiprows=1, index_col=0).astype(float)
    ones = df.multiply(df.index.size).divide(np.arange(df.index.size)+1, axis=0)
    result = final_function(ones)

    atleast = 9
    sinking = get_series_with_at_least_k_valid_values(result.sinking, atleast)
    rising = get_series_with_at_least_k_valid_values(result.rising, atleast - 6)
    print(sinking)
    print(rising)


def final_function(ones):
    df = create_blocks(ones)
    sinking, sink = create_sinking_df(df)
    r_f_splits = [split_columns(df, sinking, col, sink) for col in sinking]
    sinking = [x['sinking'] for x in r_f_splits if 'sinking' in x.keys()]
    sinking = pd.DataFrame(list(chain.from_iterable(sinking)))
    sinking.name = 'sinking'

    rising = [x['rising'] for x in r_f_splits if 'rising' in x.keys()]
    rising = pd.DataFrame(list(chain.from_iterable(rising)))
    rising.name = 'rising'

    result = namedtuple('result', 'rising, sinking')
    return result(rising.T, sinking.T)


def create_blocks(ones):
    criteria = ones > .5
    groups = criteria
    groupclusters = (groups.shift(1) != groups).astype(int).cumsum()

    # Rename columns
    for col in groupclusters:
        groupclusters[col+'b'] = groupclusters[col]
        groupclusters.drop(col, inplace=True, axis=1)
    ones_with_blocks = pd.concat([criteria, groupclusters], axis=1)

    # Calculate columns
    total_columns = 0
    for col in ones:
        dframe = ones_with_blocks.reset_index().groupby([col,col+'b'])['ix'].apply(np.array)
        total_columns += dframe.size
    total_columns

    # Create blocks
    blocks = np.ones((ones.index.size,total_columns))*np.nan
    blocks = pd.DataFrame(blocks, columns=["c{:03d}".format(x) for x in range(total_columns)])

    ix = 0
    get_rid = []
    for col in ones:
        dframe = ones_with_blocks.reset_index().groupby([col,col+'b'])['ix'].apply(np.array)
        for indices in dframe:
            s = [x for x in range(indices.size)]
            blocks["c{:03d}".format(ix)][s] = ones[col][indices].values
            ix+=1

    for col in blocks:
        i = 0
        while i < blocks.index.size:
            shifted = blocks.loc[:,col].shift(-i)
            if shifted[0] < 1 and shifted[0] > 0:
                blocks.loc[:,col] = blocks.loc[:,col].shift(-i)
                break
            i+=1
        if i == 32:
            blocks.drop(col, inplace=True, axis=1)

    for col in blocks:
        if (blocks[col][:2] > .5).all():
            blocks[col] = 1 - blocks[col]
        elif blocks[col][0] > .5 and np.isnan(blocks[col][1]):
            blocks[col] = 1 - blocks[col]

    assert (blocks.iloc[0,:] > .5).sum() == 0, "Woooohooooo///"
    return blocks


def create_sinking_df(df):
    sinking = (df.shift(1) >= df).astype(int)  # 1 if it is sinking
    sinking.iloc[0,:] = sinking.iloc[1,:]
    return sinking, 1


def split_columns(blocks, sinks, col, sink):
    result = dict()
    sink = 1
    for (bit, df) in sinks[col].reset_index().groupby(col):
        indices = df['index'].index.values
        if bit == sink:
            result['sinking'] = split_and_fill(blocks[col], indices)
        else:
            result['rising'] = split_and_fill(blocks[col], indices)
    return result


def split_and_fill(series, indices):
    splits = split_by_continues_behaviour(indices)
    result = add_nans(series, splits)
    return result


def split_by_continues_behaviour(indices):
    splits = []
    subset = [indices[0]-1, indices[0]] if indices[0] != 0 else [indices[0]]
    for v in indices[1:]:
        if np.isnan(v):
            break
        if v == subset[-1]+1:
            subset.append(v)
        else:
            splits.append(subset)
            subset = [v-1, v]
    splits.append(subset)
    return splits


def add_nans(series, splits):
    goal = series.size
    result = []
    for s in splits:
        data = np.ones(goal) * np.nan
        data[np.arange(len(s))] = series[s]
        result.append(data)
    return result


def get_series_with_at_least_k_valid_values(df, k):
    df = df.loc[:,df.index.size - df.isna().sum() > k]
    df.rename({x: "c{:03d}".format(x) for x in df.columns}, axis=1)
    return df


def _nan_equal(a,b):
    try:
        np.testing.assert_equal(a,b)
    except AssertionError:
        return False
    return True


if __name__ == '__main__':
    filename = '../emac.ml.tm1.f32.little.5x90x160x320_3.raw.residual.bplanes.32.csv'
    main(filename)
