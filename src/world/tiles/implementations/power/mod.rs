use std::sync::{Arc};
use parking_lot::RwLock;

pub mod generator;
pub mod lamp;

// IMPORTANT:
// ticks are ONLY sent to networks
// if a network doesnt have transformers it is autoticked, see below
// generators and consumers are NOT ticking machines, they receive updates from their network

// 3 types of tick

// 1) output ticks, machines which have their outputs get ticked (if they have a buffer with available output they also get ticked)
// This output it added to network
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

pub trait PowerGenerator {
    fn max_amperage(&self) -> Amperage;
    fn voltage(&self) -> Voltage;

    fn generate(&mut self) -> Option<Joule>;

    fn has_buffer(&self) -> bool;
    fn buffer_tick(&mut self, network: &mut PowerNetwork);

    fn has_ticked(&self) -> bool;
    fn end_tick(&mut self);
}

// Input machines (lamp, processor, etc...):
// - has buffer
// - input params like power needed, max voltage, etc...

// when getting ticked, it gets network
// try to consume correct amount of power from the network
// if insufficient power is provided, try to consume required extra from buffer
// if power still insufficient, consumption is done however task isn't executed
// finish execution and return network with consumed power taken

// during buffer tick
// get network, attempt to extract max throughput of buffer charge
// if not fully able to take power, take whatever it can and push to buffer
// any excess remains in network for other machines to charge off of

pub trait PowerConsumer {
    fn voltage(&self) -> Voltage;
    fn expected_ampergae(&self) -> Amperage;

    fn has_buffer(&self) -> bool;

    fn consume_tick(&mut self, network: &mut PowerNetwork);
    fn buffer_tick(&mut self, network: &mut PowerNetwork);

    fn has_ticked(&self) -> bool;
    fn end_tick(&mut self);
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

pub trait PowerTransformer {
    fn min_ratio(&self) -> f64;
    fn max_ratio(&self) -> f64;
    
    fn max_voltage(&self) -> Voltage;
    fn max_amperage(&self) -> Amperage;
    
    fn tick(&mut self);

    fn input_network(&self) -> Arc<RwLock<PowerNetwork>>;
    fn output_network(&self) -> Arc<RwLock<PowerNetwork>>;

    fn has_ticked(&self) -> bool;
    fn end_tick(&mut self);
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

// during non autotick, save power excess during frame, if multiple transformers, the tick wont tick generators second time, but will have the excess power available from the generators



pub struct PowerNetwork {
    current_power: Joule,

    voltage: Voltage,

    // adapt to actually work with our system
    inputs: Vec<Arc<dyn PowerGenerator>>,
    outputs: Vec<Arc<dyn PowerConsumer>>,
    input_transformers: Vec<Arc<dyn PowerTransformer>>,
    output_transformers: Vec<Arc<dyn PowerTransformer>>,
}

impl PowerNetwork {
    pub fn is_autotick(&self) -> bool {
        self.input_transformers.is_empty()
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

    pub fn tick(&mut self) {
        // for generator in &self.inputs {
        //     if !generator.has_ticked() {
        //         if let Some(power) = generator.generate() {
        //             self.current_power += power;
        //         }
        //     }
        // }
        //
        // for transformer in &self.output_transformers {
        //     if !transformer.has_ticked() {
        //         transformer.tick();
        //     }
        // }
        //
        // for output in &self.outputs {
        //     if !output.has_ticked() {
        //         output.consume_tick(self);
        //     }
        // }
        //
        // // buffer phase
        // for output in &self.outputs {
        //     if output.has_buffer() {
        //         output.buffer_tick(self);
        //     }
        // }
        //
        // for generator in &self.inputs {
        //     if generator.has_buffer() {
        //         generator.buffer_tick(self);
        //     }
        // }
    }

    pub fn end_tick(&mut self) {
        // self.current_power = 0.0;
        // for input in &self.inputs {
        //    input.end_tick();
        // }
        // for output in &self.outputs {
        //    output.end_tick();
        // }
        // for transformer in &self.output_transformers {
        //    transformer.end_tick();
        // }
    }
}