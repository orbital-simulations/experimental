# Features

## Objects

- [x] bodies 
- [ ] sensors https://github.com/orbital-simulations/experimental/issues/69

## Shapes

- [x] circle
- [x] half-plane
- [ ] convex polygon https://github.com/orbital-simulations/experimental/issues/77
- [ ] capsule https://github.com/orbital-simulations/experimental/issues/60
- [ ] composite https://github.com/orbital-simulations/experimental/issues/78

## Properties

### General properties

- [ ] type (dynamic / kinematic / static) https://github.com/orbital-simulations/experimental/issues/79

### Static properties

- [x] position
- [x] orientation

### Kinematic properties

- [x] velocity
- [x] angular velocity

### Dynamic properties

- [x] mass
- [x] inertia
- [x] force
- [x] torque
- [ ] damping https://github.com/orbital-simulations/experimental/issues/80

### Pair properties

- [ ] restitution (collision elasticity) https://github.com/orbital-simulations/experimental/issues/81
- [ ] friction https://github.com/orbital-simulations/experimental/issues/82

## Constraints

https://github.com/orbital-simulations/experimental/issues/31

- [x] contact
- [x] collision
- [x] distance
- [ ] angle https://github.com/orbital-simulations/experimental/issues/31
- [ ] translation https://github.com/orbital-simulations/experimental/issues/31
- [ ] position https://github.com/orbital-simulations/experimental/issues/31
- [ ] velocity https://github.com/orbital-simulations/experimental/issues/31
- [ ] limits https://github.com/orbital-simulations/experimental/issues/83
- [ ] motors https://github.com/orbital-simulations/experimental/issues/84
- [ ] ragdoll https://github.com/orbital-simulations/experimental/issues/1

# Simulation

- [x] integration
- [x] gravity
- [x] contact handling
- [x] discrete collision detection
- [x] constraint solving
- [ ] inelastic collisions https://github.com/orbital-simulations/experimental/issues/53
- [ ] damping https://github.com/orbital-simulations/experimental/issues/80
- [ ] sleeping https://github.com/orbital-simulations/experimental/issues/85
- [ ] friction https://github.com/orbital-simulations/experimental/issues/11
- [ ] stable stacking https://github.com/orbital-simulations/experimental/issues/86
- [ ] continuous collision detection https://github.com/orbital-simulations/experimental/issues/87

# API

## Object management

- [x] direct data manipulation
- [ ] handle-based CRUD https://github.com/orbital-simulations/experimental/issues/88
- [ ] impulse / force registration (should this be a separate API from CRUD?) https://github.com/orbital-simulations/experimental/issues/89

## Simulation management

- [x] delta time step
- [x] basic configuration
- [ ] sub-stepping https://github.com/orbital-simulations/experimental/issues/90

## Events

- [ ] contact https://github.com/orbital-simulations/experimental/issues/106
- [ ] collision https://github.com/orbital-simulations/experimental/issues/106
- [ ] sensor https://github.com/orbital-simulations/experimental/issues/106

## Queries

- [ ] ray casts https://github.com/orbital-simulations/experimental/issues/96
- [ ] shape overlap https://github.com/orbital-simulations/experimental/issues/97

# Tools

## Inspector

- [x] basic playground with examples
- [ ] save & load scenes https://github.com/orbital-simulations/experimental/issues/98
- [ ] edit scenes https://github.com/orbital-simulations/experimental/issues/99

## Record & replay

- [ ] record https://github.com/orbital-simulations/experimental/issues/100
- [ ] replay (in the inspector?) https://github.com/orbital-simulations/experimental/issues/101

## Profiling

- [ ] gather stats https://github.com/orbital-simulations/experimental/issues/102
- [ ] visualizing slow objects https://github.com/orbital-simulations/experimental/issues/103

# Implementation

## Integration

- [x] semi-implicit Euler
- [ ] Verlet https://github.com/orbital-simulations/experimental/issues/104
- [ ] Runge--Kutta https://github.com/orbital-simulations/experimental/issues/105

## Discrete collision detection

### Broad phase

- [ ] research acceleration structures https://github.com/orbital-simulations/experimental/issues/24
- [ ] bounding-box-based tree https://github.com/orbital-simulations/experimental/issues/91
- [ ] spatial hashing https://github.com/orbital-simulations/experimental/issues/92

### Narrow phase

- [x] direct circle/circle test
- [x] direct circle/half-plane test
- [ ] direct tests for other shapes https://github.com/orbital-simulations/experimental/issues/60
- [ ] separating axis theorem (SAT) https://github.com/orbital-simulations/experimental/issues/71
- [ ] Gilbert--Johnson--Keerthi (GJK) https://github.com/orbital-simulations/experimental/issues/70
- [ ] Minkowski portal refinement (MPR) https://github.com/orbital-simulations/experimental/issues/72
- [ ] Expanding polytope algorithm (EPA) https://github.com/orbital-simulations/experimental/issues/73

## Continuous collision detection

- [ ] time of impact calculation https://github.com/orbital-simulations/experimental/issues/93

## Constraint solving

- [x] sequential impulse solver
- [ ] projected Gauss--Seidel https://github.com/orbital-simulations/experimental/issues/94
- [ ] linear complementarity problem (LCP) solvers https://github.com/orbital-simulations/experimental/issues/95
