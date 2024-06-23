//! Uses DMA to copy memory to memory.
//!

//% FEATURES: log debug
//% CHIPS: esp32s3

#![no_std]
#![no_main]

use esp32s3 as pac;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    dma::{Dma, DmaPriority, Mem2Mem},
    dma_buffers,
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
};
use log::{error, info};

#[entry]
fn main() -> ! {
    esp_println::logger::init_logger(log::LevelFilter::Debug);
    info!("main starting!!");

    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let delay = Delay::new(&clocks);

    const DATA_SIZE: usize = 4092;
    let (tx_buffer, mut tx_descriptors, mut rx_buffer, mut rx_descriptors) =
        dma_buffers!(DATA_SIZE, DATA_SIZE);

    let dma = Dma::new(peripherals.DMA);
    let channel = dma.channel0.configure(
        false,
        &mut tx_descriptors,
        &mut rx_descriptors,
        DmaPriority::Priority0,
    );

    let mut mem2mem = Mem2Mem::new(channel);

    for i in 0..core::mem::size_of_val(tx_buffer) {
        tx_buffer[i] = (i % 256) as u8;
    }

    info!("Starting transfer");
    let result = mem2mem.start_transfer(&tx_buffer, &mut rx_buffer);
    match result {
        Ok(dma_wait) => {
            info!("Transfer started");
            dma_wait.wait().unwrap();
            info!("Transfer completed");

            for i in 0..core::mem::size_of_val(tx_buffer) {
                if rx_buffer[i] != tx_buffer[i] {
                    error!(
                        "Error: tx_buffer[{}] = {}, rx_buffer[{}] = {}",
                        i, tx_buffer[i], i, rx_buffer[i]
                    );
                    break;
                }
            }
        }
        Err(e) => {
            error!("start_transfer: Error: {:?}", e);
            let r = unsafe { &*pac::DMA::ptr() };
            info!("{:?}", r.ch(0).in_conf0());
            info!("IN_INT_RAW: {:?}", r.ch(0).in_int().raw());
            info!("{:?}", r.ch(0).in_link());
            info!("{:?}", r.ch(0).out_conf0());
            info!("{:?}", r.ch(0).in_pri());
            info!("OUT_INT_RAW: {:?}", r.ch(0).out_int().raw());
            info!("{:?}", r.ch(0).out_link());
            info!("{:?}", r.ch(0).outfifo_status());
            info!("{:?}", r.ch(0).out_state());
            info!("{:?}", r.ch(0).out_eof_des_addr());
            info!("{:?}", r.ch(0).out_eof_bfr_des_addr());
            info!("{:?}", r.ch(0).out_dscr());
            info!("{:?}", r.ch(0).out_dscr_bf0());
            info!("{:?}", r.ch(0).out_dscr_bf1());
            info!("{:?}", r.ch(0).out_pri());
            info!("{:?}", r.extmem_reject_st());
            info!("{:?}", r.misc_conf());
            drop(result);
            info!("TX buffer: {:p}", tx_buffer.as_ptr());
            info!("RX buffer: {:p}", rx_buffer.as_ptr());
            info!(
                "TX desc: addr: {:p} {:?}",
                tx_descriptors.as_ptr(),
                &tx_descriptors
            );
            info!(
                "RX desc: addr: {:p} {:?}",
                rx_descriptors.as_ptr(),
                &rx_descriptors
            );
        }
    }

    loop {
        delay.delay(2.secs());
    }
}
