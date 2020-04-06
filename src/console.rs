//! Print debug information to UART0
//!
//! Directly writes to the UART0 TX uart queue.
//! This is unsafe! It is asynchronous with normal UART0 usage and
//! interrupts are not disabled.

use esp32::UART0;
use crate::serial::{config::Config, NoRx, NoTx, Serial};
use crate::dport::Split;
use crate::serial::config::{DataBits, Parity, StopBits};

pub struct DebugLog {}

pub enum Error {}

impl DebugLog {
    pub fn begin(baud:u32){
        let dp = unsafe { esp32::Peripherals::steal() };

        let (mut dport, dport_clock_control) = dp.DPORT.split();

        let clkcntrl = crate::clock_control::ClockControl::new(
            dp.RTCCNTL,
            dp.APB_CTRL,
            dport_clock_control,
            crate::clock_control::XTAL_FREQUENCY_AUTO,
        )
            .unwrap();

        let (clkcntrl_config, _watchdog) = clkcntrl.freeze().unwrap();

        let _serial = Serial::uart0(
            dp.UART0,
            (NoTx, NoRx),
            Config {
                baudrate: crate::units::Hertz(baud),
                data_bits: DataBits::DataBits8,
                parity: Parity::ParityNone,
                stop_bits: StopBits::STOP1,
            }, // default configuration is 19200 baud, 8 data bits, 1 stop bit & no parity (8N1)
            clkcntrl_config,
            &mut dport,
        ).unwrap();
    }

    pub fn count(&mut self) -> u8 {
        unsafe { (*UART0::ptr()).status.read().txfifo_cnt().bits() }
    }

    pub fn is_idle(&mut self) -> bool {
        unsafe { (*UART0::ptr()).status.read().st_utx_out().is_tx_idle() }
    }

    pub fn write(&mut self, byte: u8) -> nb::Result<(), Error> {
        if self.count() < 128 {
            unsafe { (*UART0::ptr()).tx_fifo.write_with_zero(|w| w.bits(byte)) }
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl core::fmt::Write for DebugLog {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes()
            .iter()
            .try_for_each(|c| nb::block!(self.write(*c)))
            .map_err(|_| core::fmt::Error)
    }
}

pub static mut DEBUG_LOG: DebugLog = DebugLog {};

/// Macro for sending a formatted string to UART0 for debugging
#[macro_export]
macro_rules! dprint {
    ($s:expr) => {
        unsafe {$crate::console::DEBUG_LOG.write_str($s).unwrap()};
    };
    ($($arg:tt)*) => {
        unsafe {$crate::console::DEBUG_LOG.write_fmt(format_args!($($arg)*)).unwrap()};
    };
}

/// Macro for sending a formatted string to UART0 for debugging, with a newline.
#[macro_export]
macro_rules! dprintln {
    () => {
        unsafe {$crate::console::DEBUG_LOG.write_str("\n").unwrap()};
    };
    ($fmt:expr) => {
        unsafe {$crate::console::DEBUG_LOG.write_str(concat!($fmt, "\n")).unwrap()};
    };
    ($fmt:expr, $($arg:tt)*) => {
        unsafe {$crate::console::DEBUG_LOG.write_fmt(format_args!(concat!($fmt, "\n"), $($arg)*)).unwrap()};
    };
}

/// Macro for sending a formatted string to UART0 for debugging, with a newline.
#[macro_export]
macro_rules! dflush {
    () => {
        unsafe { while !$crate::console::DEBUG_LOG.is_idle() {} };
    };
}
