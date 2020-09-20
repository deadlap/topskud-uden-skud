use crate::{game::DELTA};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Energy {
    pub cur_energy: f32,
    pub max_energy: f32,
    pub energy_regain: f32,
}

impl Energy {
    pub fn try_to_use_energy(&mut self, amount: f32) -> bool {
        if self.cur_energy < amount {
            false
        } else {
            self.cur_energy -= amount;
            true
        }
    }

    pub fn update(&mut self) {
        if (self.max_energy-self.cur_energy).abs() > 0.  {
            let energy_update = self.energy_regain * DELTA;
            if self.max_energy > self.cur_energy+energy_update {
                self.cur_energy += energy_update;
            } else {
                self.cur_energy = self.max_energy;
            }
        }
    }
}

impl Default for Energy {
    fn default() -> Self {
        Self {
            max_energy: 100.,
            cur_energy: 100.,
            energy_regain: 7.5,
        }
    }
}