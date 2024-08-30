#!/usr/bin/env python3

import matplotlib.pyplot as plt
from matplotlib import gridspec
import networkx as nx
from networkx.drawing.nx_pydot import graphviz_layout
import json
import sys


def load_snapshots(fname):
    snapshots = []
    with open(fname) as fp:
        for line in fp.readlines():
            line = line.strip()
            if len(line) != 0:
                snapshot = json.loads(line)
                snapshots.append(snapshot)
    return snapshots


class SimRes():
    def __init__(self, fname):
        self.fname = fname
        self.snapshots = load_snapshots(fname)

    def draw_snapshot(self, i):
        snapshot = self.snapshots[i]
        secs = snapshot["running_duration"]["secs"]
        uavs = snapshot["uavs"]
        fig = plt.figure(figsize=plt.figaspect(0.5))  # set up a figure twice as wide as it is tall
        fig.suptitle(f"{secs}s", fontsize=16)
        ax0 = fig.add_subplot(1, 2, 1)
        self.draw_tree_figure(uavs, ax0)
        ax1 = fig.add_subplot(1, 2, 2, projection='3d')
        self.draw_pos_figure(uavs, ax1)
        plt.show()

    def draw_tree_figure(self, uavs, ax, tree_prog="dot", tree_ns=300, tree_fs=12, **kwargs):
        idset = {uav["nid"][-1] for uav in uavs}
        graph = nx.DiGraph()
        for uav in uavs:
            nid = uav["nid"]
            id = nid[-1]
            graph.add_node(id)
            if 1 < len(nid):
                parent = nid[-2]
                if parent in idset:
                    graph.add_edge(parent, id)
        pos = graphviz_layout(graph, prog=tree_prog)  # some other prog: "twopi", "circo", ...
        nx.draw(graph, pos=pos, ax=ax, with_labels=True, arrows=True, arrowstyle='-|>',
                node_size=tree_ns, font_size=tree_fs)
        ax.set_title("tree structure")

    def draw_pos_figure(self, uavs, ax, pos_ms=15, pos_text=True, pos_fs=10,
                        pos_view_elev=30, pos_view_azim=45,
                        pos_xlim=(0.0, 20.0), pos_ylim=(0.0, 20.0), pos_zlim=(0.0, 20.0),
                        pos_xticks=(0, 5, 10, 15), pos_yticks=(0, 5, 10, 15), pos_zticks=(0, 5, 10, 15, 20),
                        **kwargs):
        xs = [uav["p"]["x"] for uav in uavs]
        ys = [uav["p"]["y"] for uav in uavs]
        zs = [uav["p"]["z"] for uav in uavs]
        ax.scatter(xs, ys, zs, s=pos_ms, marker='o')
        for uav in uavs:
            id = uav["nid"][-1]
            pos = uav["p"]
            if pos_text:
                ax.text(pos["x"], pos["y"], pos["z"], str(id), size=pos_fs, zorder=1, color='k')
        ax.set_xlabel('x(m)')
        ax.set_ylabel('y(m)')
        ax.set_zlabel('z(m)')
        ax.set_xlim(pos_xlim)
        ax.set_ylim(pos_ylim)
        ax.set_zlim(pos_zlim)
        ax.set_xticks(pos_xticks)
        ax.set_yticks(pos_yticks)
        ax.set_zticks(pos_zticks)
        ax.view_init(elev=pos_view_elev, azim=pos_view_azim, roll=0)
        ax.set_title("UAV positions")


def draw_sim_figures(sim: SimRes, snapshots: list, sav_prefix="", **kwargs):
    nshots = len(snapshots)
    figs = []
    for idx, ishot in enumerate(snapshots):
        snapshot = sim.snapshots[ishot]
        secs = snapshot["running_duration"]["secs"] + snapshot["running_duration"]["nanos"] * 1.0e-9
        uavs = snapshot["uavs"]
        fig = plt.figure(figsize=plt.figaspect(0.5))  # set up a figure twice as wide as it is tall
        fig.suptitle(f"simulation time: {secs:.1f}s", fontsize=15)
        ax0 = fig.add_subplot(1, 2, 1)
        sim.draw_tree_figure(uavs, ax0, **kwargs)
        ax1 = fig.add_subplot(1, 2, 2, projection='3d')
        sim.draw_pos_figure(uavs, ax1, **kwargs)
        fig.savefig(sav_prefix + f".{ishot:02d}.png")
        figs.append(fig)
    return figs


def draw_line_figures():
    sim = SimRes("data/050ms-0.10m/out-line-20240828-215415")
    return draw_sim_figures(sim, [1, 5, 6, 11, 12, 15], sav_prefix="line")


def draw_lttr_figures():
    sim = SimRes("data/050ms-0.10m/out-lttr-20240828-221140")
    return draw_sim_figures(sim, [1, 4, 7, 8, 12, 13, 14, 18],
                            sav_prefix="lttr",
                            tree_prog="twopi",  # "twopi" or "circo"
                            tree_ns=100, tree_fs=8,
                            pos_view_elev=15, pos_view_azim=5,
                            pos_ms=12, pos_text=False, pos_fs=8,
                            pos_xlim=(0.0, 25.0), pos_ylim=(0.0, 25.0), pos_zlim=(0.0, 25.0),
                            pos_xticks=(0, 10, 20), pos_yticks=(0, 10, 20), pos_zticks=(0, 10, 20))


if __name__ == "__main__":
    if len(sys.argv) == 2:
        sim = SimRes(sys.argv[1])
        ishot = int(sys.argv[2])
        sim.draw_snapshot(ishot)
    else:
        draw_line_figures()
        draw_lttr_figures()
        plt.show()