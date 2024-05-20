const MACHINE_CAPACITY: u32 = 1000;

pub struct Machine {
    pool: u32
}

impl Machine {
    pub fn with(initial_pool: u32) -> Result<Machine, &'static str> {
        if initial_pool > MACHINE_CAPACITY {
            return Err("too many coins");
        }

        Ok(Machine { pool: initial_pool })
    }

    pub fn get_pool(&self) -> u32 {
        self.pool
    }

    // probabilidad de que caigan monedas
    fn p(&self) -> f64 {
        // stepness
        let k: f64 = 0.02;

        // threshhold
        let t: f64 = 700.0;

        let n: f64 = f64::from(self.pool);

        1.0_f64/(1.0_f64 + (-k * (n - t)).exp())
    }
}


#[cfg(test)]
mod machine_tests {
    use super::*;

    #[test]
    fn create_machine_with_max_coins() {
        let m = Machine::with(MACHINE_CAPACITY);

        assert!(m.is_ok());
        assert_eq!(m.unwrap().get_pool(), MACHINE_CAPACITY);
    }

    #[test]
    fn create_machine_with_too_many_coins() {
        let m = Machine::with(MACHINE_CAPACITY + 1);

        assert!(m.is_err());
    }

    #[test]
    fn prob_with_one_coin() {
        let m = Machine::with(1).unwrap();

        let r = m.p();

        // Si solo hay una moneda, la prob de que caigan
        // debe ser menor a 1 en 1 millon
        assert!(r < 0.000001_f64);
    }

    #[test]
    fn prob_with_max_coins() {
        let m = Machine::with(MACHINE_CAPACITY).unwrap();

        let r = m.p();

        // Si las monedas estan al máximo, la prob de que caigan
        // debe ser mayor a 99%
        assert!(r > 0.99_f64);
    }
}
