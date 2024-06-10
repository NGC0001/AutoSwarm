# Coordinated UAV Fleet Route Planning

## Background

In many situations, a system is composed of many homogeneous or heterogeneous
subsystems. These subsystems need to be coordinated so as to complete certain tasks,
e.g., starlink constellation, power distribution stations in a grid, etc.

Currently, such systems are usually manually coordiated, commonly by a centralised controller.
It is extremely difficult to decentralise and fully automate the coordination.

For my project, I hope to focus on unmanned aerial vehicle (UAV, or drone) fleet,
and to develop an algorithm for autonomously coordinated route planning.
UAVs are usually controlled by a ground station, or they fly by pre-programmed paths.
UAV fleet will be more flexible and versatile
if UAVs can work out their routes dynamically on their own in a coordinated way.
Such fleet will be useful in many situations. For example:

- Aerial forest fire fighting;
- Post-disaster search and rescue;
- Aerial combat of UAV swarm;
- Drone shows.

I think this is still quite a difficult topic.
Especially if general-case task planning is considered, a lot of theories may be involved.
With certian presumptions, things can be easier.

## Basic presumptions

- The task is for the UAVs to form a given geometric shape;
- UAVs shall work out their final positions and flight paths;
- UAVs are homogeneous, that is, they know each other well and behave the same;
- Good communications are established, so no need to deal with latency, etc;
- No obstacles, except that UAVs shall not collide with each other;
- There may be limitations on usable airspace;
- UAV can get its location through GPS.

The focus is path planning algorithm, vehicle dynamics won't be considered.
The algorithm can be tested by simulation.

## Tech stack

I'd like to develop under Linux enviroment.
Rust is prefered programming language. However, currently I'm not familiar with Rust.
If things get tough or if necessary libraries are missing,
I might turn to C++ instead, which I'm relatively good at.

## Expected output

I hope at least to develop a software with well-defined interfaces, possibly used by DIY drones.
Beyond that, if feasible, I wish to do some research on coordinated task planning and task division,
and maybe to publish a paper if possible.

## Open to Other Ideas

I'm also open to other project ideas. For example:

- Large scale simulation;
- Computer architectures;
- Operating systems;
- Game engines;
- Other high-performance systems.
