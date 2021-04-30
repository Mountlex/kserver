#!/usr/bin/python3

import argparse
import sys
import os
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

DETERMINISTIC_ALG_SERVER = "Double-Coverage"
DETERMINISTIC_ALG_TAXI = "BiasedDC"
ENHANCED_ALG_SERVER = "LDC"
DET_COMBINE_ALG_SERVER = "Det_Combination"
ENHANCED_ALG_TAXI = "LBDC"


def create_arg_parser():
    parser = argparse.ArgumentParser(
        description='Plot results from kserver simulation')
    parser.add_argument('sampleFile')
    parser.add_argument('-b', '--bin_size', default=0.25)
    parser.add_argument('-l', '--lambdas', nargs="+", default=[0.0, 0.2, 0.4, 0.6, 0.8, 1.0])
    parser.add_argument('-k', '--number_of_servers', default=2)
    parser.add_argument('--max', action='store_true')

    return parser


def get_data(filename):
    data = pd.read_csv(filename)
    #data = data.round(3)
    data['EtaOverOpt'] = data['Eta'] / data['OptCost']
    data['CRalg'] = data['LDC'] / data['OptCost']
    data['FtP&DC'] = data['RobustFtp'] / data['OptCost']
    data['DC'] = data['DC'] / data['OptCost']
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
    #dfCombine = df.loc[:, ['Lmbda', 'FtP & DC', 'Bin']]
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
       # ax = dfCombine.groupby(['Lmbda']).mean().plot(ax=ax,
       #     label="Det_Combine", linewidth=2, legend=True)

    for label, l in list(grouped_data):
        grouped_data[(label, l)].plot(ax=ax, linewidth=2,
                                      style='--', label=f"{pred_alg} (η/Opt<={l:1.1f})", legend=True)

    plt.plot((0, 1), (1, 1), 'black')

    #x = np.arange(0.005, 1.0, 0.005)
    #plt.plot(x, cons(x), 'r', label='Consistency')
    #plt.plot(x, robust(x), 'r', label='Robustness')
    plt.xlabel('Lambda')
    plt.ylabel('Empirical competitive ratio')
    #plt.axis([0, 1, 0.9, 1.1])

    if args.max:
        plt.title(
            f"Simulation with {args.number_of_servers} servers (max over all samples)")
    else:
        plt.title(
            f"Simulation with {args.number_of_servers} servers (mean over all samples)")

    fig = plt.gcf()
    plt.legend(loc="upper right",
               bbox_transform=fig.transFigure, ncol=2)
    fig.set_dpi(250)
    fig.set_size_inches(14, 8, forward=True)
    # fig.subplots_adjust(right=0.7)


def plot_eta(df, eta_res, args, pred_alg):
    df = df[df['Lmbda'].isin(args.lambdas)]
    df['Bin'] = np.ceil(df['EtaOverOpt'] / eta_res) * eta_res
    max_bin = df['Bin'].max()
    dfAlg = df.loc[:, ['Lmbda', 'CRalg', 'Bin']]
    dfcombine = df.loc[:, ['FtP&DC', 'Bin']]
    dfdc = df.loc[:, ['DC', 'Bin']]

    ax = None
    if args.max:
        grouped_data = dfAlg.groupby(['Bin', 'Lmbda']).max().unstack('Lmbda')
        ax = dfcombine.groupby(['Bin']).max().plot(ax=ax,
            label='Combine_det', style='s-', markersize=4, linewidth=1.2, legend=True)
    else:
        grouped_data = dfAlg.groupby(['Bin', 'Lmbda']).mean().unstack('Lmbda')
        ax = dfcombine.groupby(['Bin']).mean().plot(ax=ax,
            label='Combine_det', style='s-', markersize=4, linewidth=1.2, legend=True)
        ax = dfdc.groupby(['Bin']).mean().plot(ax=ax,
            label='DC', style='-', markersize=4, linewidth=1.2, legend=True)

    for label, l in list(grouped_data):
        grouped_data[(label, l)].plot(ax=ax,
            style='o--', markersize=4, linewidth=1.2, label=f"{pred_alg} (λ = {l:1.2f})", legend=True)



    plt.plot((0, max_bin), (1, 1), 'black')
    plt.xlabel('Eta / Opt')
    plt.ylabel('Empirical competitive ratio')
    plt.legend(loc='upper left', ncol=2)
    plt.tight_layout()
    #plt.axis([0, max_bin, 0.99, 1.1])

    if args.max:
        plt.title(
            f"{args.number_of_servers} servers")
    #else:
        #plt.title(
        #    f"{args.number_of_servers} servers")

    fig = plt.gcf()
    fig.set_dpi(500)
    fig.set_size_inches(2,1.25)
    # fig.subplots_adjust(right=0.7)
    #fig.savefig("result.png", dpi=400)


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
       # plot_lambda(data, float(parsed_args.bin_size),
       #             parsed_args, det_alg, pred_alg)
        plt.show()
    else:
        print("Path not valid!")
