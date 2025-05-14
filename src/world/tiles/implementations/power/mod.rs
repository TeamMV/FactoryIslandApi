use std::sync::{Arc};
use mvengine::event::EventBus;
use mvutils::Savable;
use mvutils::unsafe_utils::Unsafe;
use parking_lot::RwLock;
use crate::event::Event;
use crate::FactoryIsland;
use crate::world::tiles::pos::TilePos;
use crate::world::tiles::tiles::InnerTile;
use crate::world::tiles::update::{UpdateHandler, UpdateTile};
use crate::world::World;

pub mod generator;
pub mod lamp;

// IMPORTANT:
// ticks are ONLY sent to networks
// if a network doesnt have transformers it is autoticked, see below
// generators and consumers are NOT ticking machines, they receive updates from their network

// 3 types of tick

// 1) output ticks, machines which have their outputs get ticked (if they have a buffer with available output they also get ticked)
// This output is added to network
// 2) consumption tick: after output is registered into a network, consumers will receive a consumption tick, they will be allowed to consume power
// during this the machine is allowed to modify the total power in the network, both pulling and pushing power
// machines will only be able to run this consumption once per tick
// 3) buffer tick, any excess power will be pushed into a buffer if the block has the option to do that

// consuming machines with buffers will be ticked after all power ticks are executed if their buffers have spare power and they haven't already been ticked

// machine types:

// Output machines (generators, etc...):
// - has buffer?
// - output stuff like amps volts and power

// when getting ticked, they first see if their current generation is the max output they could provide
// if it is, output that max output and end execution
// if not, attempt to extract needed extra power from buffer
// if power still not enough, output partial power generation
// if no power, machine will be marked as off

// after consumption is finished any excess power can be put into buffer through its own buffer tick
// works identical to input machine buffer tick

pub type Joule = f64;
pub type Amperage = f64;
pub type Voltage = f64;

pub type NetworkId = u64;

pub trait PowerGenerator: Send + Sync {
    fn network_id(&self) -> NetworkId;

    fn max_amperage(&self) -> Amperage;
    fn voltage(&self) -> Voltage;

    fn generate(&mut self) -> Option<Joule>;

    fn has_buffer(&self) -> bool;
    fn buffer_tick(&mut self, network: &mut PowerNetwork);

    fn has_ticked(&self) -> bool;
    fn end_tick(&mut self);

    fn box_clone(&self) -> Box<dyn PowerGenerator>;
    fn save_to_vec(&self) -> Vec<u8>;
    fn load_into_self(&mut self, data: Vec<u8>);
}

// Input machines (lamp, processor, etc...):
// - has buffer
// - input params like power needed, max voltage, etc...

// when getting ticked, it gets network
// try to consume correct amount of power from the network
// if insufficient power is provided, try to consume required extra from buffer
// if power still insufficient, consumption is done however task isn't executed
// in the case that the task isn't executed, consumed power is stored and the machine will NOT count itself as ticked
// this is because a machine may be ticked again in the same tick with some extra power, which could allow it to run
// finish execution and return network with consumed power taken

// during buffer tick:
// get network, attempt to extract max throughput of buffer charge
// if not fully able to take power, take whatever it can and push to buffer
// any excess remains in network for other machines to charge off of

pub trait PowerConsumer: Send + Sync {
    fn network_id(&self) -> NetworkId;

    fn voltage(&self) -> Voltage;
    fn expected_ampergae(&self) -> Amperage;

    fn has_buffer(&self) -> bool;

    fn consume_tick(&mut self, network: &mut PowerNetwork);
    fn buffer_tick(&mut self, network: &mut PowerNetwork);

    fn has_ticked(&self) -> bool;
    fn end_tick(&mut self);

    fn box_clone(&self) -> Box<dyn PowerConsumer>;
    fn save_to_vec(&self) -> Vec<u8>;
    fn load_into_self(&mut self, data: Vec<u8>);
}

// Transformer
// - no buffer, no need to check
// - min/max ratio
// - current ratio (setting thing or smth)
// - min/max volts and amps (will explode)

// when receiving an input:
// 1) convert into output
// 2) tick non autoticked output network
// also save how much power P it has at the start
// network from output will first use all other means of power before transformer's power
// any excess power up to the original power P will be pushed back into the input network
// excess above P is left in output network incase another transformer ticks it again
// if there isn't excess nothing is returned to input network
// if there is excess below P a fraction of the power is returned to input network

pub trait PowerTransformer: Send + Sync {
    fn min_ratio(&self) -> f64;
    fn max_ratio(&self) -> f64;
    
    fn max_voltage(&self) -> Voltage;
    fn max_amperage(&self) -> Amperage;
    
    fn tick(&mut self, source: &mut PowerNetwork, world: &mut World, event_bus: &mut EventBus<Event>, fi: &FactoryIsland);

    fn input_network(&self) -> NetworkId;
    fn output_network(&self) -> NetworkId;

    fn has_ticked(&self) -> bool;
    fn end_tick(&mut self);

    fn box_clone(&self) -> Box<dyn PowerTransformer>;
    fn save_to_vec(&self) -> Vec<u8>;
    fn load_into_self(&mut self, data: Vec<u8>);
}

// Network
// - doesn't autotick? (output contains transformer)
// connected inputs
// connected outputs

// if autotick:
// during 1) outputs are ticked, all power is summed up
// in some undetermined order, inputs are ticked (consume power, can freely modify amount in network, cannot modify voltage)
// after consumption tick is over, run buffer tick if there is leftover power

// if NOT autotick:
// wait for transformer to get ticked, when it does, additionally tick all generators
// run normal system as if it was autoticked

// if end of tick and transformer didn't get ticked, run autotick method, ignore transformer

// Note that tick() doesn't check whether it has been ticked yet, as it MAY be ticked more than once. This is possible because each individual component may only be ticked once
// However components that didn't receive enough power are allowed to tick again, which allows the extra power to be used to finish their tick

// during non autotick, save power excess during frame, if multiple transformers, the tick wont tick generators second time, but will have the excess power available from the generators

#[derive(Clone, Savable)]
pub struct PowerNetwork {
    #[unsaved]
    current_power: Joule,
    #[unsaved]
    was_ticked: bool,

    id: NetworkId,
    voltage: Voltage,

    inputs: Vec<TilePos>,
    outputs: Vec<TilePos>,
    input_transformers: Vec<TilePos>,
    output_transformers: Vec<TilePos>,
}

impl PowerNetwork {
    pub fn is_autotick(&self) -> bool {
        self.input_transformers.is_empty()
    }

    pub fn was_ticked(&self) -> bool {
        self.was_ticked
    }

    pub fn id(&self) -> NetworkId {
        self.id
    }

    pub fn voltage(&self) -> Voltage {
        self.voltage
    }

    pub fn current_power(&self) -> Joule {
        self.current_power
    }

    pub fn consume_power(&mut self, power: Joule) -> bool {
        self.current_power -= power;
        self.current_power >= 0.0
    }

    pub fn insert_power(&mut self, power: Joule) {
        self.current_power += power;
    }

    pub fn tick(&mut self, world: &mut World, event_bus: &mut EventBus<Event>, fi: &FactoryIsland) {
        self.was_ticked = true;
        for pos in self.inputs.clone() {
            if let Some(generator) = world.get_tile_at(pos, event_bus) {
                let mut lock = generator.write();
                if let InnerTile::PowerGenerator(generator) = &mut lock.info.inner {
                    if !generator.has_ticked() {
                        if let Some(power) = generator.generate() {
                            self.current_power += power;
                        }
                    }
                }
            }
        }

        for pos in self.output_transformers.clone() {
            if let Some(transformer) = world.get_tile_at(pos, event_bus) {
                let mut lock = transformer.write();
                if let InnerTile::PowerTransformer(transformer) = &mut lock.info.inner {
                    let transformer = unsafe { Unsafe::cast_mut_static(transformer) };
                    drop(lock);
                    if !transformer.has_ticked() {
                        transformer.tick(self, world, event_bus, fi);
                    }
                }
            }
        }

        for pos in self.outputs.clone() {
            if let Some(consumer) = world.get_tile_at(pos, event_bus) {
                let mut lock = consumer.write();
                if let InnerTile::PowerConsumer(consumer) = &mut lock.info.inner {
                    if !consumer.has_ticked() {
                        consumer.consume_tick(self);
                    }
                }
            }
        }

        // Buffer phase
        for pos in self.outputs.clone() {
            if let Some(consumer) = world.get_tile_at(pos, event_bus) {
                let mut lock = consumer.write();
                if let InnerTile::PowerConsumer(consumer) = &mut lock.info.inner {
                    if consumer.has_buffer() {
                        consumer.buffer_tick(self);
                    }
                }
            }
        }

        for pos in self.inputs.clone() {
            if let Some(generator) = world.get_tile_at(pos, event_bus) {
                let mut lock = generator.write();
                if let InnerTile::PowerGenerator(generator) = &mut lock.info.inner {
                    if generator.has_buffer() {
                        generator.buffer_tick(self);
                    }
                }
            }
        }
    }

    pub fn end_tick(&mut self) {
        self.was_ticked = false;
    }
}