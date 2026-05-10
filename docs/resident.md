# Resident system

Residents are individual entities as the population of the colony.

## Location

In terms of location, a resident is always in one of the following states:

- In the ambient space of a building, located by 3D coordinates within the building.
- In the ambient space of a corridor, located by 1D coordinates within the corridor measuring distance from endpoints.
- In a [vehicle](vehicle.md).
- In a facility with class `resident::InteractSlots`.

The resident interacts with the ambient fluid of the building or corridor they are located in,
or the fluid of the vehicle they are in.
Even if the resident is interacting with a facility, they interact with the ambient fluid of the building hosting the facility,
not the fluid of the facility itself, even if the facility has its own fluid storage.

## Movement

### Intra-building/corridor movement

Residents can move within buildings at a standard speed in 3D space subject to their physical attributes.
The basic speed is called the standard walking speed.

Residents can also move within corridors, but cross-sectional area is not considered; only the length of the corridor matters.

### Boarding/alighting vehicles

Residents can board or alight vehicles when they are in the same building as the vehicle,
and the vehicle speed is less than standard walking speed.

### Entering facilities

Residents can enter a facility with class `resident::InteractSlots`
when they are near the center of the building hosting the facility.

When residents leave a facility, they start at the center of the building.

## Facility interaction

A facility with class `resident::InteractSlots` defines a list of interaction slots,
which can be claimed by residents to interact with the facility.
Each slot has headcount limit.
Some facilities require residents to occupy interaction slots to be usable,
while others apply effects to residents occupying these slots.

## Attributes

Attributes are quantitative properties of residents affecting their behavior.
Some interaction slots require a minimum or maximum attribute value to be claimed.

Residents interacting with a facility may affect building efficiency subject to their attributes.
In particular, some "skill" attributes are specific to certain jobs.

Some facilities also affect the attributes of residents in return.
In particular, some [reactors](reactor.md) behave like "schools" that increase certain skill attributes.

## Cargo carrying

Residents can carry a small amount of cargo with them,
which can be loaded from or unloaded to any facility or vehicle with `cargo::Storage` class
when they are located in the same building (not corridors).

## Survival

"Hitpoints" is a core attribute represent the physical integrity of a resident.
When hitpoints drop to zero, the resident dies immediately and despawns.
They drop carried cargo as well as a "corpse" cargo item at the location of death.
The corpse cargo item is volatile and liberates harmful fluids,
negatively affecting the health and happiness of nearby residents.

Death also significantly damages the happiness of residents they have high psychological affinity with.

### Breathing

Residents consume oxygen from their interacting fluid, and produce carbon dioxide in return.
Critically low oxygen concentration would result in hitpoint loss.

Some attributes are affected by oxygen concentration or combos.
Example use cases:

- A disease mod may add a "respiratory disease" attribute
  that raises the oxygen concentration threshold for hitpoint loss.
- Temporary strength attributes may be limited by oxygen concentration.

### Other builtin attributes

Attributes are largely customized by mods.
The following defines some default setups:

- Health represents the general physical well-being of a resident.
  The maximum health degrades with age.
  When health is critically low, other attributes such as hitpoints are reduced.
  It also has impacts on skill attributes defined as health-dependent.
- Hunger is represented as a simple attribute that increases over time,
  and may be reduced by interacting with [reactors](reactor.md) that modify the hunger attribute.

## Behavior

Resident behavior is controlled by a hierarchy of AIs.
Some AIs are selectable by the player,
while others automatically override player control under specific circumstances.
Earlier layers override or affect later layers.

### 0. Survivability

The base survivability AI overrides a resident to seek improvement of critical attributes:

- When hitpoints and other attributes marked as "survival" are critically low,
  the resident searches for nearby facilities that can restore these attributes and tries to interact with them.
- When oxygen concentration is critically low,
  the resident tries to move to a different location with higher oxygen concentration.

### 1. Fatigue

"Fatigue" is a core attribute representing the unwillingness of a resident to comply with player commands.
Consistent work increases fatigue, which can be restored gradually over time or by interacting with specific facilities.

When fatigue is high, the resident would stop accepting player instructions.
The fatigue AI would try to reduce resident fatigue
by accessing facilities and fluids that reduce fatigue.

### 2. Crime

"Morality" is a core attribute that determines whether a resident would commit crimes.
Extreme physical attributes damage morality, which can only be restored through specific education.

When morality is low, the resident would be controlled by a crime AI that commits crimes.
Crime AI behavior includes:

- Theft: transferring cargo from storages or another player's inventory
- Assault: damaging hitpoints of other residents
- Vandalism: damaging facilities or vehicles

Crimes are triggered based on specific needs.
For example, a resident with constantly low happiness may favor assault,
low physiological attributes may favor theft, and high fatigue may favor vandalism.

Crimes are executed opportunistically and may involve higher-level planning.
For example, assault may bias towards residents with weaker physical attributes;
theft may bias towards areas with lower surveillance.

Mods can add hooks to increase resident attributes such as infamy when crimes are committed,
combined with other affinities such as surveillance.

### 3. Restraint

Residents with "restraint" cargo in their inventory may restrain another resident
if certain attribute conditions are satisfied (i.e. one can physically overpower the other).
Restrained residents are controlled by restraint AI that would follow a restrainer to board or alight vehicles.

Restrained residents can be transported to detention facilities.
Detention is not a builtin concept, but can be implemented manually
by sealing off the ambient space connections to prevent escape by foot
and limiting the vehicles entering the detention area to only those that require special authorization to operate.

The restrainer is controlled by the Patrol AI below,
identifying the restrained resident based on affinities such as infamy and surveillance.

### 4. Occupation

The game computes a pool of "jobs" available for residents to work on
based on the facilities and vehicles that require interaction.
Each job has a priority and attribute affinities.
A resident chooses the job with the highest combination of priority and attribute affinity score to work on.

### 5. Patrol

Some jobs involve patrolling between multiple locations on a vehicle.
The patrol AI is an umbrella of algorithms that execute these jobs.
Examples include:

- Construction: finds a construction vehicle and moves to the construction site.
- Maintenance: actively moving to facilities that require maintenance.
- Security: actively patrol low-surveillance areas or areas with previous crime,
  or chase after identified criminals.
- Janitor: actively moving to cargo items that require cleanup, such as corpses,
  and transporting them to specific storage/treatment facilities.
- Transportation: driving vehicles such as buses to transport residents/cargo along fixed routes.

### 6. Housing

The housing AI ensures residents return home regularly as configured by player expectations,
resulting in a work-life cycle involving daily commute.

The housing AI also actively identifies high-intimacy (see below) residents
and tries to house them together, optionally favoring fertile couples to increase the chance of natural birth.

### 7. Pathfinding

The previous layers of AI determine the target location for a resident to move to.
The pathfinding AI computes the optimal path to the target location and moves the resident along the path,
based on world topology and real-time traffic conditions such as vehicle timetables.

## Intimacy

Each resident has a "intimacy compatibility" represented as a random 128-bit mask.
High morality residents may randomly gain a new bit (set to 1), increasing their intimacy compatibility.
Low morality residents may randomly lose a bit (set to 0), decreasing their intimacy compatibility.

When two residents are in the same building or vehicle,
their pairwise intimacy is increased at a rate proportional to the
number of ones in the bitwise AND of their intimacy masks.
It is impossible for two residents to have intimacy if they do not share any common bit in their intimacy masks.

Intimacy results in psychological damage when the other is dead.

## Housing

Facilities with class `resident::Housing` provide interaction slots for housing.
Each slot headcount has a growing "privacy" attribute,
which is reset every time a different resident occupies the slot.
A resident restores happiness when staying in a slot with higher privacy.

Residents with high pairwise intimacy would receive bonus happiness
when their housing slots are in the same building.

## Birth

There are two sources of new residents:

### Stored embryos

An embryo reserve facility cannot be constructed by the player
and is typically provided as part of the initial colony setup.

It provides the player with base residents to start with as well as a backup source in case of population collapse.
Activating embryos requires specific resources, after which a new resident is spawned.

### Natural birth

When two residents have high intimacy and have opposite gender,
the female one has a chance to become pregnant when they are in the same building,
subject to their pairwise fertility.
Pregnancy is a core attribute that gradually increases once positive.
Once it reaches a threshold, it must be treated as a facility with class `resident::Birthplace`.
Failure to do so (including inactive facility) would result in
severe hitpoint damage to the pregnant resident.
On the contrary, successful treatment would result in reset of pregnancy
and creation of a new resident.

The new child resident shares random 64 bits of intimacy from each parent.

### Genetics

Each resident also has a "genetic code" represented as a random 4096-bit mask.
The genetic code of naturally born residents inherit random 2048 bits of genetic code from each parent,
with a small chance of random mutation (flipping a random bit).

The pairwise fertility of two residents is determined by the number of ones in the bitwise XOR of their genetic code.
Couples with more ones are more likely to be fertile,
encouraging moral diversity and preventing inbreeding.
