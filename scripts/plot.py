import argparse
import sys
import os
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

DETERMINISTIC_ALG_SERVER = "Double-Coverage"
DETERMINISTIC_ALG_TAXI = "BiasedDC"
ENHANCED_ALG_SERVER = "LDC"
ENHANCED_ALG_TAXI = "LBDC"


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
    #data = data.round(3)
    data['EtaOverOpt'] = data['Eta'] / data['OptCost']
    data['CRalg'] = data['AlgCost'] / data['OptCost']
    data['CRdc'] = data['DcCost'] / data['OptCost']
    return data


def cons(x):
    return 1+x


def robust(x):
    return 1 + 1/x


def plot_lambda(df, eta_res, args, det_alg, pred_alg):
    df['Bin'] = np.ceil(df['EtaOverOpt'] / eta_res) * eta_res
    df[det_alg] = df['CRdc']
    # df = df[df['Bin'] < 10]
    dfAlg = df.loc[:, ['Lmbda', 'CRalg', 'Bin']]
    dfDC = df.loc[:, ['Lmbda', det_alg]]

    if args.max:
        grouped_data = dfAlg.groupby(
            ['Bin', 'Lmbda']).max().unstack('Bin')
        ax = dfDC.groupby(['Lmbda']).max().plot(
            label=det_alg, linewidth=2, legend=True)
    else:
        grouped_data = dfAlg.groupby(
            ['Bin', 'Lmbda']).mean().unstack('Bin')
        ax = dfDC.groupby(['Lmbda']).mean().plot(
            label=det_alg, linewidth=2, legend=True)

    for label, l in list(grouped_data):
        grouped_data[(label, l)].plot(ax=ax, linewidth=2,
                                      style='--', label=f"{pred_alg} (Eta/Opt ={l:1.2f})", legend=True)

    plt.plot((0, 1), (1, 1), 'black')

    #x = np.arange(0.005, 1.0, 0.005)
    #plt.plot(x, cons(x), 'r', label='Consistency')
    #plt.plot(x, robust(x), 'r', label='Robustness')
    plt.xlabel('Lambda')
    plt.ylabel('Empirical competitive ratio')
    #plt.axis([0, 1, 0.9, 2.5])

    if args.max:
        plt.title(
            f"Simulation with {args.number_of_servers} servers (max over all samples)")
    else:
        plt.title(
            f"Simulation with {args.number_of_servers} servers (mean over all samples)")

    fig = plt.gcf()
    plt.legend(loc="lower right",
               bbox_transform=fig.transFigure, ncol=2)
    fig.set_dpi(250)
    fig.set_size_inches(14, 8, forward=True)
    # fig.subplots_adjust(right=0.7)


def plot_eta(df, eta_res, args, pred_alg):
    lambdas = list(np.linspace(0, 1, num=args.lambdas))
    lambdas = [round(l, 3) for l in lambdas]
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
            style='--', linewidth=2, label=f"{pred_alg} (Lambda = {l:1.2f})", legend=True)

    plt.plot((0, max_bin), (1, 1), 'black')
    plt.xlabel('Eta / Opt')
    plt.ylabel('Empirical competitive ratio')
    plt.legend(loc='lower right')

    if args.max:
        plt.title(
            f"Simulation with {args.number_of_servers} servers (max over all samples)")
    else:
        plt.title(
            f"Simulation with {args.number_of_servers} servers (mean over all samples)")

    fig = plt.gcf()
    fig.set_dpi(250)
    fig.set_size_inches(10, 8, forward=True)
    # fig.subplots_adjust(right=0.7)


if __name__ == "__main__":
    arg_parser = create_arg_parser()
    parsed_args = arg_parser.parse_args(sys.argv[1:])
    if os.path.exists(parsed_args.sampleFile):
        print(parsed_args.bin_size)
        if "taxi" in parsed_args.sampleFile:
            det_alg = DETERMINISTIC_ALG_TAXI
            pred_alg = ENHANCED_ALG_TAXI
        else:
            det_alg = DETERMINISTIC_ALG_SERVER
            pred_alg = ENHANCED_ALG_SERVER

        data = get_data(parsed_args.sampleFile)
        plot_eta(data, float(parsed_args.bin_size),
                 parsed_args, pred_alg)
        plot_lambda(data, float(parsed_args.bin_size),
                    parsed_args, det_alg, pred_alg)
        plt.show()
    else:
        print("Path not valid!")
