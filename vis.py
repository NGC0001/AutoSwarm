#!/usr/bin/env python3

import matplotlib.pyplot as plt
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
        uavs = snapshot["uavs"]
        xs = [uav["p"]["x"] for uav in uavs]
        ys = [uav["p"]["y"] for uav in uavs]
        zs = [uav["p"]["z"] for uav in uavs]
        fig = plt.figure()
        ax = fig.add_subplot(projection="3d")
        ax.scatter(xs, ys, zs, marker='o')
        ax.set_xlabel('x')
        ax.set_ylabel('y')
        ax.set_zlabel('z')
        ax.set_xlim(0.0, 10.0)
        ax.set_ylim(10.0, 20.0)
        ax.set_zlim(5.0, 15.0)
        for uav in uavs:
            id = uav["nid"][-1]
            pos = uav["p"]
            ax.text(pos["x"], pos["y"], pos["z"], str(id), size=10, zorder=1, color='k')
        plt.show()


if __name__ == "__main__":
    if len(sys.argv) == 2:
        sim = SimRes(sys.argv[1])
        sim.draw_snapshot(12)
    else:
        raise RuntimeError("need exact one argument as data file")