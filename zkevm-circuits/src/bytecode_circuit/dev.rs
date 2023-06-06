pub use super::circuit::BytecodeCircuit;

use crate::{
    bytecode_circuit::circuit::{BytecodeCircuitConfig, BytecodeCircuitConfigArgs},
    table::{BytecodeTable, KeccakTable},
    util::{Challenges, SubCircuit, SubCircuitConfig},
};
use eth_types::Field;
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{Circuit, ConstraintSystem, Error},
};

impl<F: Field> Circuit<F> for BytecodeCircuit<F> {
    type Config = (BytecodeCircuitConfig<F>, Challenges);
    type FloorPlanner = SimpleFloorPlanner;
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let bytecode_table = BytecodeTable::construct(meta);
        let keccak_table = KeccakTable::construct(meta);
        let challenges = Challenges::construct(meta);

        let config = {
            let challenges = challenges.exprs(meta);
            BytecodeCircuitConfig::new(
                meta,
                BytecodeCircuitConfigArgs {
                    bytecode_table,
                    keccak_table,
                    challenges,
                },
            )
        };

        (config, challenges)
    }

    fn synthesize(
        &self,
        (config, challenges): Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        use std::fs::OpenOptions;
        use std::io::prelude::*;
        use std::time::{Instant, Duration};
        
        println!("Start challenge timer");
        let timer_challenge = Instant::now();  // start timer
        let challenges = challenges.values(&mut layouter);
        let duration_challenge = timer_challenge.elapsed();  // end timer

        println!("Start keccak timer");
        let timer_keccak = Instant::now();  // start timer
        config.keccak_table.dev_load(
            &mut layouter,
            self.bytecodes.iter().map(|b| &b.bytes),
            &challenges,
        )?;
        let duration_keccak = timer_keccak.elapsed();  // end timer

        println!("Start synthesize sub timer");
        let timer_synthesize_sub = Instant::now();  // start timer
        self.synthesize_sub(&config, &challenges, &mut layouter)?;
        let duration_synthesize_sub = timer_synthesize_sub.elapsed();  // end timer
        
        let duration_total = duration_challenge + duration_keccak + duration_synthesize_sub;
        println!("Total time elapsed: {:?}", duration_total);
        let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("original_timer_result.txt")?;
        writeln!(file, "    Synthesize TOTAL {:?}", duration_total)?;
        writeln!(file, "        challenge {:?}", duration_challenge)?;
        writeln!(file, "        keccak table {:?}", duration_keccak)?;
        writeln!(file, "        synthesize_sub {:?}", duration_synthesize_sub)?;

        Ok(())
    }
}
