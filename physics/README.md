# Features

## Objects

- [x] bodies
- [ ] sensors https://github.com/orbital-simulations/experimental/issues/69

## Shapes

- [x] circle
- [x] half-plane
- [ ] convex polygon
- [ ] capsule https://github.com/orbital-simulations/experimental/issues/60
- [ ] composite

## Properties

### General properties

- [ ] type (dynamic / kinematic / static)

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
- [ ] damping

### Pair properties

- [ ] restitution (collision elasticity)
- [ ] friction

## Constraints

https://github.com/orbital-simulations/experimental/issues/31

- [x] contact
- [x] collision
- [x] distance
- [ ] angle
- [ ] translation
- [ ] position
- [ ] velocity
- [ ] limits
- [ ] motors
- [ ] ragdoll https://github.com/orbital-simulations/experimental/issues/1

# Simulation

- [x] integration
- [x] gravity
- [x] contact handling
- [x] discrete collision detection
- [x] constraint solving
- [ ] inelastic collisions https://github.com/orbital-simulations/experimental/issues/53
- [ ] damping
- [ ] sleeping
- [ ] friction https://github.com/orbital-simulations/experimental/issues/11
- [ ] stable stacking
- [ ] continuous collision detection

# API

## Object management

- [x] direct data manipulation
- [ ] handle-based CRUD
- [ ] impulse / force registration (should this be a separate API from CRUD?)

## Simulation management

- [x] delta time step
- [x] basic configuration
- [ ] sub-stepping

## Events

- [ ] contact 
- [ ] collision 
- [ ] sensor 

## Queries

- [ ] ray casts
- [ ] shape overlap

# Tools

## Inspector

- [x] basic playground with examples
- [ ] save & load scenes
- [ ] edit scenes

## Record & replay

- [ ] record
- [ ] replay (in the inspector?)

## Profiling

- [ ] gather stats
- [ ] visualizing slow objects

# Implementation

## Integration

- [x] semi-implicit Euler
- [ ] Verlet
- [ ] Runge--Kutta

## Discrete collision detection

### Broad phase

- [ ] research acceleration structures https://github.com/orbital-simulations/experimental/issues/24
- [ ] bounding-box-based tree
- [ ] spatial hashing

### Narrow phase

- [x] direct circle/circle test
- [x] direct circle/half-plane test
- [ ] direct tests for other shapes https://github.com/orbital-simulations/experimental/issues/60
- [ ] separating axis theorem (SAT) https://github.com/orbital-simulations/experimental/issues/71
- [ ] Gilbert--Johnson--Keerthi (GJK) https://github.com/orbital-simulations/experimental/issues/70
- [ ] Minkowski portal refinement (MPR) https://github.com/orbital-simulations/experimental/issues/72
- [ ] Expanding polytope algorithm (EPA) https://github.com/orbital-simulations/experimental/issues/73

## Continuous collision detection

- [ ] time of impact calculation 

## Constraint solving

- [x] sequential impulse solver
- [ ] projected Gauss--Seidel
- [ ] linear complementarity problem (LCP) solvers
