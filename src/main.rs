// on our power chip, we use the LAN 8720A with RMII Interface

/*
We use an external 25MHz crystal source for the chip. Because of this, the esp-idf docs say:
In this case, you should select CONFIG_ETH_RMII_CLK_INPUT in CONFIG_ETH_RMII_CLK_MODE.
CONFIG_ETH_RMII_CLK_MODE

 So our sdkconfig should contain
 CONFIG_ETH_USE_ESP32_EMAC=y (the default is yes)
 CONFIG_ETH_RMII_CLK_MODE=CONFIG_ETH_RMII_CLK_INPUT
 CONFIG_ETH_PHY_INTERFACE=CONFIG_ETH_PHY_INTERFACE_RMII

Note that it can be changed in user code.

Our max data rate is 10/100 Ethernet (10Base-T or 100BASE-TX)


The PHY should be held in reset state (N_RESET on our board).

The RMII interface connects to the ESP32 Ethernet MAC .

The MDI interface connects to the Transformer where the RJ45 is.


*/

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::{gpio, peripherals::Peripherals};
use esp_idf_svc::sys::EspError;
fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();

    let sysloop = EspSystemEventLoop::take().unwrap();
    FreeRtos::delay_ms(1000);
    log::info!("Getting ethernet!");

    FreeRtos::delay_ms(1000);
    let pins = peripherals.pins;
    // let rst = PinDriver::output(pins.gpio5).unwrap();

    let mac = peripherals.mac;

    let driver = esp_idf_svc::eth::EthDriver::new_rmii(
        mac,
        pins.gpio25, //rxd0
        pins.gpio26, //rxd1
        pins.gpio27, // crs
        pins.gpio23, // mdc
        pins.gpio22, // txd1
        pins.gpio21, // tx_en
        pins.gpio19, //txd0
        pins.gpio18, //mdio
        esp_idf_svc::eth::RmiiClockConfig::<gpio::Gpio0, gpio::Gpio16, gpio::Gpio17>::Input(
            pins.gpio0,
        ),
        Some(pins.gpio5),
        esp_idf_svc::eth::RmiiEthChipset::LAN87XX,
        None,
        sysloop.clone(),
    );
    if let Err(driv_err) = driver {
        log::warn!("Driver error :{:?}", driv_err);
        return;
    }
    let mut eth = esp_idf_svc::eth::EspEth::wrap(driver.unwrap()).unwrap();
    log::warn!("Driver init ");
    eth_configure(&sysloop, &mut eth).unwrap();
    log::warn!("Configure done ");
    // eth.start().unwrap();
    // return eth;

    println!("Done!");
    FreeRtos::delay_ms(100000);
}

fn eth_configure<'d, T>(
    sysloop: &EspSystemEventLoop,
    eth: &mut esp_idf_svc::eth::EspEth<'d, T>,
) -> Result<(), EspError> {
    let mut eth = esp_idf_svc::eth::BlockingEth::wrap(eth, sysloop.clone())?;
    log::warn!("Start ");
    eth.start()?;
    log::warn!("Wait for it to be up ");
    eth.wait_netif_up()?;

    let ip_info = eth.eth().netif().get_ip_info()?;

    println!("ip info = {:?}", ip_info);

    Ok(())
}
