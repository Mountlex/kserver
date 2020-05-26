import argparse
import sys
import os
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt


def create_arg_parser():
    parser = argparse.ArgumentParser(
        description='Plot results from kserver simulation')
    parser.add_argument('sampleFile')
    parser.add_argument('-b', '--bin_size', default=0.25)
    parser.add_argument('-l', '--lambdas', default=5, type=int)
    parser.add_argument('-k', '--number_of_servers', default=2)
    parser.add_argument('--max', action='store_true')

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


def plot_lambda(df, eta_res, args):
    df['Bin'] = np.ceil(df['EtaOverOpt'] / eta_res) * eta_res
    df['Double-Coverage'] = df['CRdc']
    # df = df[df['Bin'] < 10]
    dfAlg = df.loc[:, ['Lmbda', 'CRalg', 'Bin']]
    dfDC = df.loc[:, ['Lmbda', 'Double-Coverage']]

    if args.max:
        grouped_data = dfAlg.groupby(
            ['Bin', 'Lmbda']).max().unstack('Bin')
        ax = dfDC.groupby(['Lmbda']).max().plot(
            label='DoubleCoverage', legend=True)
    else:
        grouped_data = dfAlg.groupby(
            ['Bin', 'Lmbda']).mean().unstack('Bin')
        ax = dfDC.groupby(['Lmbda']).mean().plot(
            label='DoubleCoverage', legend=True)

    for label, l in list(grouped_data):
        grouped_data[(label, l)].plot(ax=ax,
                                      style='--', label=f"LambdaDC (Eta/Opt <= {l:1.2f})", legend=True)

    plt.plot((0, 1), (1, 1), 'black')

    #x = np.arange(0.005, 1.0, 0.005)
    #plt.plot(x, cons(x), 'r', label='Consistency')
    #plt.plot(x, robust(x), 'r', label='Robustness')
    plt.xlabel('Lambda')
    plt.ylabel('Empirical competitive ratio')
    #plt.axis([0, 1, 0.9, 2.5])
    plt.legend(loc='center left', bbox_to_anchor=(1, 0.5))

    plt.title(f"Simulation with {args.number_of_servers} servers")

    fig = plt.gcf()
    fig.set_dpi(200)
    fig.set_size_inches(15, 8, forward=True)
    fig.subplots_adjust(right=0.7)


def plot_eta(df, eta_res, args):
    lambdas = list(np.linspace(0, 1, num=args.lambdas))
    print(lambdas)
    df = df[df['Lmbda'].isin(lambdas)]
    df['Bin'] = np.ceil(df['EtaOverOpt'] / eta_res) * eta_res
    max_bin = df['Bin'].max()
    dfAlg = df.loc[:, ['Lmbda', 'CRalg', 'Bin']]

    if args.max:
        grouped_data = dfAlg.groupby(['Bin', 'Lmbda']).max().unstack('Lmbda')
    else:
        grouped_data = dfAlg.groupby(['Bin', 'Lmbda']).mean().unstack('Lmbda')

    for label, l in list(grouped_data):
        grouped_data[(label, l)].plot(
            style='--', label=f"LambdaDC (Lambda = {l:1.2f})", legend=True)

    plt.plot((0, max_bin), (1, 1), 'black')
    plt.xlabel('Eta / Opt')
    plt.ylabel('Empirical competitive ratio')
    plt.legend(loc='center left', bbox_to_anchor=(1, 0.5))
    plt.title(f"Simulation with {args.number_of_servers} servers")

    fig = plt.gcf()
    fig.set_dpi(200)
    fig.set_size_inches(15, 8, forward=True)
    fig.subplots_adjust(right=0.7)


if __name__ == "__main__":
    arg_parser = create_arg_parser()
    parsed_args = arg_parser.parse_args(sys.argv[1:])
    if os.path.exists(parsed_args.sampleFile):
        print(parsed_args.bin_size)
        data = get_data(parsed_args.sampleFile)
        plot_eta(data, float(parsed_args.bin_size), parsed_args)
        plot_lambda(data, float(parsed_args.bin_size), parsed_args)
        plt.show()
    else:
        print("Path not valid!")
