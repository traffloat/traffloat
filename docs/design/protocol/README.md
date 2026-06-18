# Protocol

Updates from the physics world are represented in two types:
"meta" defines gamerule information,
while "viewable" defines volatile information that can be (un)subscribed on demand.

- Graph fixtures
  - Buildings
  - Corridors (edges are represented as properties of corridors)
  - Facilities (connections are represented as properties of facilities)
  - Pipes
- Resident
- Vehicle

## Viewable

When a viewer is spawned (e.g. a multiplayer client joins the game),
they declare the subscription config desired.
The following options are available:

- `Normal`: what a normal player should see, subject to game rules
- `Debug`: reveals all information possible when focused on a viewable
- `Scraper`: syncs all information about all viewables, never unsubscribe until despawn

A viewer forms a pairwise subscription relation with each viewable,
quantified by a "subscription level".
Unlike subscription config, subscription level is different for each viewer-viewable pair
and is determined by the subscription config plus environmental conditions:

- None: the viewer does not know about the existence of the viewable at all
- `Optical`: a coarse view that only defines how to render the viewable
- `Detail`: in addition to optical information, also exposes all other information normal players can see
- `Debug`: reveals all information possible
