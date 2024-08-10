#!/usr/bin/env python3

import matplotlib.pyplot as plt
import networkx as nx
from networkx.drawing.nx_pydot import graphviz_layout
import json
import pydot
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

    def draw_tree_figure(self, uavs, ax):
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
        pos = graphviz_layout(graph, prog="dot")  # some other prog: "twopi", "circo", ...
        nx.draw(graph, pos=pos, ax=ax, with_labels=True, arrows=True, arrowstyle='-|>')

    def draw_pos_figure(self, uavs, ax):
        xs = [uav["p"]["x"] for uav in uavs]
        ys = [uav["p"]["y"] for uav in uavs]
        zs = [uav["p"]["z"] for uav in uavs]
        ax.scatter(xs, ys, zs, marker='o')
        ax.set_xlabel('x')
        ax.set_ylabel('y')
        ax.set_zlabel('z')
        ax.set_xlim(0.0, 20.0)
        ax.set_ylim(0.0, 20.0)
        ax.set_zlim(0.0, 20.0)
        for uav in uavs:
            id = uav["nid"][-1]
            pos = uav["p"]
            ax.text(pos["x"], pos["y"], pos["z"], str(id), size=10, zorder=1, color='k')


if __name__ == "__main__":
    if len(sys.argv) == 2:
        sim = SimRes(sys.argv[1])
        sim.draw_snapshot(12)
    else:
        raise RuntimeError("need exact one argument as data file")