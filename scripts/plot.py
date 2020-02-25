import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

data = pd.read_csv('../result.csv')

data['EtaOverOpt'] = data['Eta'] / data['OptCost']
data['CRalg'] = data['AlgCost'] / data['OptCost']
data['CRdc'] = data['DcCost'] / data['OptCost']


def plot_lambda(df, eta_res):
    df['Bin'] = np.ceil(df['EtaOverOpt'] / eta_res)
    dfAlg = df.loc[:, ['Lmbda', 'CRalg', 'Bin']]
    dfDC = df.loc[:, ['Lmbda', 'CRdc']]
    ax = dfAlg.groupby(
        ['Bin', 'Lmbda']).mean().unstack('Bin').plot(style='b--', legend=False)
    dfDC.groupby(['Lmbda']).mean().plot(ax=ax)
    plt.plot((0, 1), (1, 1), label='OPT')
    plt.xlabel('Lambda')
    plt.ylabel('Competitive ratio')
    plt.legend


def plot_eta(df, eta_res):
    df['Bin'] = np.ceil(df['EtaOverOpt'] / eta_res) * eta_res
    max_bin = df['Bin'].max()
    dfAlg = df.loc[:, ['Lmbda', 'CRalg', 'Bin']]
    dfDC = df.loc[:, ['CRdc', 'Bin']]
    ax = dfAlg.groupby(
        ['Bin', 'Lmbda']).mean().unstack('Lmbda').plot(style='b--',
                                                       legend=False)
    dfDC.groupby(['Bin']).mean().plot(ax=ax)
    plt.plot((0, max_bin), (1, 1), label='OPT')
    plt.xlabel('Eta / Opt')
    plt.ylabel('Competitive ratio')


plot_eta(data, 0.25)
plot_lambda(data, 0.25)
plt.show()
