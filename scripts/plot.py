import argparse
import sys
import os
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt


PLOT_MEAN = 0
PLOT_MAX = 1


def create_arg_parser():
    parser = argparse.ArgumentParser(
        description='Plot results from kserver simulation')
    parser.add_argument('sampleFile')
    parser.add_argument('-b', '--bin_size', default=0.25)
    parser.add_argument('--max', action='store_const',
                        const=1, default=0, dest='mode')
    return parser


def get_data(filename):
    data = pd.read_csv(filename)
    data['EtaOverOpt'] = data['Eta'] / data['OptCost']
    data['CRalg'] = data['AlgCost'] / data['OptCost']
    data['CRdc'] = data['DcCost'] / data['OptCost']
    return data


def cons(x):
    return 1+x


def robust(x):
    return 1 + 1/x


def plot_lambda(df, eta_res, mode):
    df['Bin'] = np.ceil(df['EtaOverOpt'] / eta_res) * eta_res
    # df = df[df['Bin'] < 10]
    dfAlg = df.loc[:, ['Lmbda', 'CRalg', 'Bin']]
    dfDC = df.loc[:, ['Lmbda', 'CRdc']]
    if mode == PLOT_MAX:
        ax = dfAlg.groupby(
            ['Bin', 'Lmbda']).max().unstack('Bin').plot(style='--', legend=True)
        dfDC.groupby(['Lmbda']).max().plot(ax=ax)

    if mode == PLOT_MEAN:
        ax = dfAlg.groupby(
            ['Bin', 'Lmbda']).mean().unstack('Bin').plot(style='--', legend=True)
        dfDC.groupby(['Lmbda']).mean().plot(ax=ax)

    plt.plot((0, 1), (1, 1), label='OPT')

    x = np.arange(0.005, 1.0, 0.005)

    plt.plot(x, cons(x), 'r', label='Consistency')
    plt.plot(x, robust(x), 'r', label='Robustness')
    plt.xlabel('Lambda')
    plt.ylabel('Competitive ratio')
    plt.axis([0, 1, 0.9, 2.5])
    plt.legend(loc='center left', bbox_to_anchor=(1, 0.5))

    fig = plt.gcf()
    fig.set_dpi(100)
    fig.set_size_inches(15, 8, forward=True)
    fig.subplots_adjust(right=0.7)


def plot_eta(df, eta_res):
    df['Bin'] = np.ceil(df['EtaOverOpt'] / eta_res) * eta_res
    max_bin = df['Bin'].max()
    dfAlg = df.loc[:, ['Lmbda', 'CRalg', 'Bin']]
    dfDC = df.loc[:, ['CRdc', 'Bin']]
    ax = dfAlg.groupby(
        ['Bin', 'Lmbda']).mean().unstack('Lmbda').plot(style='--', label='test', legend=True)

    dfDC.groupby(['Bin']).mean().plot(ax=ax)
    plt.plot((0, max_bin), (1, 1), label='OPT')
    plt.xlabel('Eta / Opt')
    plt.ylabel('Competitive ratio')
    plt.legend(loc='center left', bbox_to_anchor=(1, 0.5))

    fig = plt.gcf()
    fig.set_dpi(100)
    fig.set_size_inches(15, 8, forward=True)
    fig.subplots_adjust(right=0.7)


if __name__ == "__main__":
    arg_parser = create_arg_parser()
    parsed_args = arg_parser.parse_args(sys.argv[1:])
    if os.path.exists(parsed_args.sampleFile):
        print(parsed_args.bin_size)
        data = get_data(parsed_args.sampleFile)
        plot_eta(data, float(parsed_args.bin_size))
        plot_lambda(data, float(parsed_args.bin_size), parsed_args.mode)
        plt.show()
    else:
        print("Path not valid!")
