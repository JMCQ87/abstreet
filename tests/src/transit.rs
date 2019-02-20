use crate::runner::TestRunner;
use abstutil::Timer;
use sim;
use sim::{Event, SimFlags, Tick};

pub fn run(t: &mut TestRunner) {
    t.run_slow("bus_reaches_stops", |h| {
        let (map, mut sim) = sim::load(
            SimFlags::for_test("bus_reaches_stops"),
            Some(Tick::from_seconds(30)),
            &mut Timer::throwaway(),
        );
        let route = map.get_bus_route("49").unwrap();
        let buses = sim.seed_bus_route(route, &map);
        let bus = buses[0];
        h.setup_done(&sim);

        let mut expectations: Vec<Event> = Vec::new();
        // TODO assert stuff about other buses as well, although the timing is a little unclear
        for stop in route.stops.iter().skip(1) {
            expectations.push(Event::BusArrivedAtStop(bus, *stop));
            expectations.push(Event::BusDepartedFromStop(bus, *stop));
        }

        sim.run_until_expectations_met(&map, expectations, Tick::from_minutes(10));
        sim.run_until_done(&map, |_| {}, Some(sim::Tick::from_minutes(20)));
    });

    t.run_slow("ped_uses_bus", |h| {
        let (map, mut sim) = sim::load(
            SimFlags::for_test("ped_uses_bus"),
            Some(Tick::from_seconds(30)),
            &mut Timer::throwaway(),
        );
        let route = map.get_bus_route("49").unwrap();
        let buses = sim.seed_bus_route(route, &map);
        let bus = buses[0];
        let ped_stop1 = route.stops[1];
        let ped_stop2 = route.stops[2];
        // TODO These should be buildings near the two stops. Programmatically find these?
        let start_bldg = *map
            .get_l(map.get_bs(ped_stop1).sidewalk_pos.lane())
            .building_paths
            .last()
            .unwrap();
        // TODO Goal should be on the opposite side of the road from the stop, but that's hard to
        // express right now. :\
        let goal_bldg = map
            .get_l(map.get_bs(ped_stop2).sidewalk_pos.lane())
            .building_paths[0];
        let ped =
            sim.seed_trip_using_bus(start_bldg, goal_bldg, route.id, ped_stop1, ped_stop2, &map);
        h.setup_done(&sim);

        sim.run_until_expectations_met(
            &map,
            vec![
                sim::Event::PedReachedBusStop(ped, ped_stop1),
                sim::Event::BusArrivedAtStop(bus, ped_stop1),
                sim::Event::PedEntersBus(ped, bus),
                sim::Event::BusDepartedFromStop(bus, ped_stop1),
                sim::Event::BusArrivedAtStop(bus, ped_stop2),
                sim::Event::PedLeavesBus(ped, bus),
                sim::Event::PedReachedBuilding(ped, goal_bldg),
                sim::Event::BusDepartedFromStop(bus, ped_stop2),
                sim::Event::BusArrivedAtStop(bus, route.stops[3]),
            ],
            sim::Tick::from_minutes(8),
        );
    });
}
