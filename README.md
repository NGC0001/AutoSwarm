# Decentralised Task Planning for Unmanned Aerial Vehicle Swarm

## Abstract

In many situations, a system is composed of many homogeneous or heterogeneous
subsystems. These subsystems need to be coordinated so as to complete certain tasks,
e.g., power distribution stations in a grid, starlink constellation, etc.
Currently, such systems are usually manually coordiated, commonly by a centralised controller.
It is extremely difficult to fully automate and decentralise the coordination.

One particular type of such systems is unmanned aerial vehicle (UAV) swarms.
Traditionally, UAV swarms are controlled manually by ground control stations (GCSs),
or they just fly by pre-programmed paths.
Some papers have proposed algorithms to automate and decentralise swarm control
[1] [2] [3] [4]. And this remains a frontier of research.
Autonomous UAV swarms will be more flexible and versatile.
Such swarms will be useful in many situations. For example:

- Aerial forest fire fighting;
- Post-disaster search and rescue;
- Aerial combat of UAV swarm;
- Drone shows.

In this project, I will develop an algorithm for autonomous decentralised swarm task planning,
in particular, path planning.
For now, I have not formulated a concrete idea about how the algorithm should work,
but only vague concepts.
I take the drone show as the specific situation to be considered.
A group of drones are instructed to form a particular geometric shape in a particular position.
The algorithm will then have to solve the following two problems

- let the drones devide tasks among themselves,
i.e., which drone should be allocated to which position;
- let each drone find a viable flying path to its allocated position,
without colliding with other drones.

## (Fuzzy) Problem Statement

A group of hovering UAVs at their initial positions.
UAVs can control their speed (, or, to make it a bit harder, their acceleration).
A mathematical function describing a geometric shape.
Within finite time, all the UAVs should be positioned on that shape,
and should be distributed as evenly as possible.

Some presumptions may be

- UAVs are homogeneous, that is, they know each other well and behave the same;
- good communications are established, so no need to deal with latency, etc;
- no obstacles, except that UAVs shall not collide with each other;
- there may be limitations on usable airspace;
- UAV can get its real-time locations precisely.

## Tech stack

I'd like to develop under Linux enviroment.
Rust is prefered programming language.
However, currently I'm not familiar with Rust.
If things get tough or if necessary libraries are missing,
I might turn to C++/Python instead, which I'm relatively good at.

## Expected output

I hope at least to develop a software implementing the algorithm with well-defined interfaces.

Beyond that, if time permits, I wish to do some research on decentralised task planning,
and maybe to publish a paper if I've gotten some novel idea.

## Test and Evaluation

The algorithm can be tested by numerical simulation,
which is the general evalution method I've seen in previous papers of this kind.

As the evalution is not expected to involve any other individuals or animals,
**ethics** review is therefore not needed.

## Time Plan

This project should be finished within 3 months.
Below are my estimated time plan

1. Research and idea formation: 0.5 - 1 month.
2. Algorithm development: 1 - 1.5 month.
3. Evalution: 0.5 - 1 month.
