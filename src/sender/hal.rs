//! Embedded-hal based Sender

use crate::sender::{ProtocolEncoder, PulsedataSender, PulseDataStatus};

/// Embedded hal sender
/// The BUFSIZE is specified in terms of pulse sender commands, each command is a single unit,
/// the required number of commands is protocol dependent
/// the NEC16 can be up to 68 pulse sender commands per protocol command
pub struct Sender<PwmPin, const FREQ: u32, const BUFSIZE: usize> {
    pin: PwmPin,
    counter: u32,
    pulsedata_sender: PulsedataSender<BUFSIZE>,
    status: SenderStatus
}

impl<PwmPin, PwmDuty, const F: u32, const S: usize> Sender<PwmPin, F, S>
where
    PwmPin: embedded_hal::PwmPin<Duty = PwmDuty>,
{
    pub fn new(pin: PwmPin) -> Self {
        Self {
            pin,
            counter: 0,
            pulsedata_sender: PulsedataSender::new(),
            status: SenderStatus::Idle
        }
    }

    pub fn load<Proto>(&mut self, cmd: &Proto::Cmd)
    where
        Proto: ProtocolEncoder<F>,
    {
        self.pulsedata_sender.load_command::<Proto, F>(cmd);
        self.counter = 0;
        self.status = SenderStatus::Transmit;
    }

    pub fn ready(&self) -> bool { self.status == SenderStatus::Idle }

    pub fn buffer(&self) -> &[u32] {
        self.pulsedata_sender.buffer()
    }

    /// Method to be called periodically to update the pwm output
    pub fn tick(&mut self) {
        if self.status == SenderStatus::Transmit {
            let status = self.pulsedata_sender.tick(self.counter);
            self.counter = self.counter.wrapping_add(1);
            match status {
                PulseDataStatus::Transmit(true) => self.pin.enable(),
                PulseDataStatus::Transmit(false) => self.pin.disable(),
                PulseDataStatus::Idle => { self.status = SenderStatus::Idle; self.pin.disable(); },
                PulseDataStatus::Error => { self.status = SenderStatus::Idle; self.pin.disable(); },
            };
        }
    }

    pub fn log_sender_status(&self) {
        defmt::info!("{} {}", self.status, self.counter);
        self.pulsedata_sender.log_state();
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SenderStatus {
    /// Sender is ready for transmitting
    #[default]
    Idle,
    /// Transmitting
    Transmit
}