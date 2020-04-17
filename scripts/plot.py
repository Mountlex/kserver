import argparse
import sys
import os
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

def create_arg_parser():
    parser = argparse.ArgumentParser(description='Plot results from kserver simulation')
    parser.add_argument('sampleFile')
    parser.add_argument('--bin_size', default=0.25)
    return parser


def get_data(filename):
    data = pd.read_csv(filename)
    data['EtaOverOpt'] = data['Eta'] / data['OptCost']
    data['CRalg'] = data['AlgCost'] / data['OptCost']
    data['CRdc'] = data['DcCost'] / data['OptCost']
    return data

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


if __name__ == "__main__":
    arg_parser = create_arg_parser()
    parsed_args = arg_parser.parse_args(sys.argv[1:])
    if os.path.exists(parsed_args.sampleFile):
        print(parsed_args.bin_size)
        data = get_data(parsed_args.sampleFile)
        plot_eta(data, parsed_args.bin_size)
        plot_lambda(data, parsed_args.bin_size)
        plt.show()
    else:
        print("Path not valid!")


