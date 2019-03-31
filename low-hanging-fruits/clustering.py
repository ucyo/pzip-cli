#!/usr/bin/env python
# coding: utf-8
"""
Clustering algorithms for correlated probabilities of binary data in files.
"""
import scipy
import numpy as np
import pandas as pd
import seaborn as sns
from matplotlib import pyplot as plt
from functools import namedtuple
from itertools import chain
from sklearn.cluster.bicluster import SpectralCoclustering, SpectralBiclustering
from matplotlib.patches import Rectangle

def main():
    filename = "emac.ml.tm1.f32.little.5x90x160x320_3.raw.residual.bplanes.32.csv"
    df = pd.read_csv(filename, skiprows=1, index_col=0).astype(float)
    corrarr = df.corr()
    clusters = building_clusters(corrarr, 'bi', n_clusters=5)

    plot_clustered_heatmap(df, clusters)
    for cluster in clusters:
        plot_slice(df, cluster)


def building_clusters(corrarr, mode, **kwargs):
    """
    Clustering of different probability strains to identify merging possibilities.

    Clustering algorithms tree:

            bi :
          /      Both algorithm use spectral clustering defined in sklearn. Need: < n_clusters >
    start - co :
          \
            easy : Uses algorithms implemented in scipy: Need: < method >, < metric >, < lvl >

    """
    assert mode in ["bi","co","easy"], "Unknown mode"
    if mode == "easy":
        necessary = ['method', 'metric', 'lvl']
        missing = [x for x in necessary if x not in kwargs.keys()]
        assert not missing, "Missing keywords {}".format(missing)
        method, metric, lvl = kwargs['method'], kwargs['metric'], kwargs['lvl']
        link = scipy.cluster.hierarchy.linkage(corrarr, method=method, metric=metric)
        result = get_members_of_lvl(link=link, lvl=lvl)
    else:
        necessary = ['n_clusters']
        missing = [x for x in necessary if x not in kwargs.keys()]
        assert not missing, "Missing keywords {}".format(missing)
        n_clusters = kwargs['n_clusters']
        result = get_spectral_clusters(corrarr, mode=mode, n_clusters=n_clusters)
    return result, [x for x in corrarr.columns]

def get_clusters(df, mode, minimum=1, **kwargs):
    min_clusters = building_clusters(cross_correlation(df, minimum), mode, **kwargs)
    clusters = map_correlation_back_to_original(df, min_clusters)
    return clusters

def cross_correlation(df, min=1):
    values_to_drop = df.corr(min_periods=min).isnull().all().values
    return df.iloc[:, ~values_to_drop].corr(min_periods=min)

def map_correlation_back_to_original(df, mclusters):
    orig_index = []
    for cluster in mclusters[0]:
        orig_index.append([list(df.columns.values).index(mclusters[1][x]) for x in cluster])
    return orig_index

def get_members_of_lvl(lvl, link):
    LVL = namedtuple("Level","members,lvl")
    clusters = [LVL(members=[x], lvl=0) for x in range(link.shape[0]+1)]
    for i in range(link.shape[0]):
        ix_1,ix_2 = int(link[i][0]), int(link[i][1])
        group = clusters[ix_1].members + clusters[ix_2].members
        max_group_number = max(clusters[ix_1].lvl,clusters[ix_2].lvl)
        clusters[ix_1] = LVL(members=clusters[ix_1].members, lvl=max_group_number)
        clusters[ix_2] = LVL(members=clusters[ix_2].members, lvl=max_group_number)
        new_lvl = max_group_number + 1
        clusters.append(LVL(members=sorted(group), lvl=new_lvl))
    maximum_lvl = clusters[-1].lvl
    clusters = [LVL(x.members,maximum_lvl-x.lvl) for x in clusters]

    selection = [x for x in clusters if x.lvl==lvl]
    lvl -= 1
    while lvl > 0:
        candidates = [x for x in clusters if x.lvl==lvl]
        winner = [x for x in candidates if not set(selection[0].members).issubset(set(x.members))]
        selection.append(winner[0])
        lvl -= 1
    return [s.members for s in selection]

def get_spectral_clusters(corrarr, n_clusters=5, mode='co'):
    if mode=='co':
        model = SpectralCoclustering(n_clusters, random_state=0)
        model.fit(corrarr)
        indices = np.arange(corrarr.columns.size)
        clusters = [indices[x].tolist() for x in model.columns_]
        return clusters
    elif mode=='bi':
        model = SpectralBiclustering(n_clusters, random_state=0)
        model.fit(corrarr)
        indices = np.arange(corrarr.columns.size)
        clusters = [indices[x].tolist() for x in model.columns_]
        repetition_start = clusters[1:].index(clusters[0]) + 1
        return clusters[:repetition_start]
    else:
        raise("Mode wrong?")

def plot_sns(df, clusters):
    mdf = get_multi_index_df(df, clusters)
    sns.scatterplot(data=mdf)
    plt.show()
    sns.lineplot(data=mdf)
    plt.show()
    colors = ['orange','red','green','skyblue']
    for i,c in enumerate(clusters):
        for v in c:
            df.iloc[:,v].plot(color=colors[i%len(colors)], style=':', marker='x')
    plt.show()

def plot_clustered_heatmap(df, clusters):
    _, ax = plt.subplots(figsize=(15,15))
    rearranged = df.iloc[:,[x for x in chain.from_iterable(clusters)]]
    sns.heatmap(rearranged.corr(), ax=ax, square=True, cbar_kws={"shrink": 0.5})

    i = 0
    start = 0
    while i < len(clusters):
        ax.add_patch(Rectangle((start, start), len(clusters[i]),
                               len(clusters[i]), fill=False, edgecolor='blue', lw=3))
        start += len(clusters[i])
        i += 1

def plot_slice(df, s, external=False):
    _, ax = plt.subplots(2, figsize=(9,10), sharex=True)
    df.iloc[:,s].plot(legend=False, ax=ax[0])
    elements = np.arange(df.index.size)+1
    if not external:
        df.iloc[:,s].multiply(df.index.size).divide(elements, axis=0).plot(legend=False, ax=ax[1])
    else:
        df.iloc[:,s].plot(legend=False, ax=ax[1])
    # plt.legend(bbox_to_anchor=(1.05, 1), loc=2, borderaxespad=0.)
    ax[0].set_title("Elements {}".format(s))
    plt.tight_layout()
    plt.show()
    plt.clf()

def get_multi_index_df(df, clusters):
    tmp = [x for x in zip(range(1, len(clusters)+1), clusters[:5])]
    clustered_index = [('C{:02}'.format(x),df.columns[y]) for x,j in tmp for y in j]

    b = list(chain.from_iterable(clusters))
    clustered_index += [('CXX', df.columns[x]) for x in range(df.columns.size) if x not in b]
    mix = pd.MultiIndex.from_tuples(clustered_index, names=['Probability Series', 'cluster'])
    cdf = pd.DataFrame(df.values, index=df.index, columns=mix)
    return cdf

if __name__ == '__main__':
    main()
