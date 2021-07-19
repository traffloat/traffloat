# Electricity
Electricity is one of the four resource types (along with cargo, liquid and gas).
Unlike the other three, electricity does not have multiple types.

## Transportation
Electricity is transferred across corridors with [power cables](../corridor/#electricity).
Unlike other resources, power cables are zero-latency.
Buildings disable randomly if it cannot receive enough electricity.

The throughput of electricity across a power cable is restricted by its radius.
Larger radius results in lower resistance hence higher throughput.

The resistance of power cables consumes electricity by generating heat.
Every unit length of power cable consumes a proportion of the electricity
passing through the cable based on the resistance.
Larger radius results in lower resistance hence lower consumption.

