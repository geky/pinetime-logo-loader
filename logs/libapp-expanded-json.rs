#![feature(prelude_import)]
#![no_std]
/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *  http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */
//!  Sensor app that reads sensor data from a temperature sensor and sends the sensor data to a CoAP server over NB-IoT.
//!  Note that we are using a patched version of apps/my_sensor_app/src/vsscanf.c that
//!  fixes AT response parsing bugs.  The patched file must be present in that location.
//!  This is the Rust version of `https://github.com/lupyuen/stm32bluepill-mynewt-sensor/blob/rust-nbiot/apps/my_sensor_app/OLDsrc/main.c`

#![no_std]
//  Don't link with standard Rust library, which is not compatible with embedded systems
#![feature(trace_macros)]
//  Allow macro tracing: `trace_macros!(true)`
#![feature(concat_idents)]
//  Allow `concat_idents!()` macro used in `coap!()` macro
#![feature(const_transmute)]
//  Allow `transmute` for initialising Mynewt structs
#![feature(proc_macro_hygiene)]
//  Allow Procedural Macros like `run!()`
#![feature(custom_attribute)]
#[prelude_import]
use ::core::prelude::v1::*;
#[macro_use]
extern crate core as core;
#[macro_use]
extern crate compiler_builtins as compiler_builtins;
//  Allow Custom Attributes like `#[safe_wrap]`

extern crate cortex_m;
//  Declare the external library `cortex_m`
extern crate mynewt;
//  Declare the Mynewt library
extern crate macros as mynewt_macros;
//  Declare the Mynewt Procedural Macros library

mod app_sensor {
    //  Declare `app_sensor.rs` as Rust module `app_sensor` for Application Sensor functions
    //  Declare `app_network.rs` as Rust module `app_network` for Application Network functions

    //  Import `PanicInfo` type which is used by `panic()` below
    //  Import cortex_m assembly function to inject breakpoint
    //  Import Mynewt OS API
    //  Import Mynewt Console API
    //  Import Mynewt Sensor Network Library

    //  Don't mangle the name "main"
    //  Declare extern "C" because it will be called by Mynewt
    //  Initialise the Mynewt packages and Blue Pill internal temperature sensor driver.
    //  Start the CoAP / OIC Background Task to transmit CoAP messages.  Any startup
    //  functions defined in pkg.yml of our custom drivers and libraries will be called by 
    //  sysinit().  Here are the startup functions consolidated by Mynewt:
    //  bin/targets/bluepill_my_sensor/generated/src/bluepill_my_sensor-sysinit-app.c


    //  Start polling the temperature sensor every 10 seconds in the background.

    //  Start the Server Transport for sending sensor data to CoAP Server over NB-IoT.

    //  Main event loop
    //  Loop forever...
    //  Processing events...
    //  From default event queue.
    //  Never comes here

    //  Display the filename and line number to the Semihosting Console.
    //  TODO: Print in decimal not hex. Allow more than 255 lines.
    //  Pause in the debugger.
    //  Loop forever so that device won't restart.
    //!  Poll the temperature sensor every 10 seconds. Transmit the sensor data to the CoAP server after polling.
    //!  This is the Rust version of https://github.com/lupyuen/stm32bluepill-mynewt-sensor/blob/rust-nbiot/apps/my_sensor_app/OLDsrc/sensor.c
    use mynewt::{result::*,
                 hw::sensor::{self, sensor_ptr, sensor_arg, sensor_data_ptr,
                              sensor_listener, sensor_temp_raw_data,
                              sensor_type_t, SensorValue, SensorValueType},
                 sys::console, fill_zero, Strn};
    use mynewt_macros::{init_strn};
    use crate::app_network::send_sensor_data;
    ///  Sensor to be polled: `temp_stm32_0` is Blue Pill's internal temperature sensor
    static SENSOR_DEVICE: Strn =
        Strn{rep: mynewt::StrnRep::ByteStr(b"temp_stm32_0\x00"),};
    ///  Poll sensor every 10,000 milliseconds (10 seconds)  
    const SENSOR_POLL_TIME: u32 = (10 * 1000);
    ///  Use key (field name) `t` to transmit raw temperature to CoAP Server
    const TEMP_SENSOR_KEY: &str = "t";
    ///  Type of sensor: Raw temperature sensor (integer sensor values 0 to 4095)
    const TEMP_SENSOR_TYPE: sensor_type_t =
        sensor::SENSOR_TYPE_AMBIENT_TEMPERATURE_RAW;
    ///  Ask Mynewt to poll the temperature sensor every 10 seconds and call `handle_sensor_data()`.
    ///  Return `Ok()` if successful, else return `Err()` with `MynewtError` error code inside.
    pub fn start_sensor_listener() -> MynewtResult<()> {
        console::print("Rust TMP poll\n");
        sensor::set_poll_rate_ms(&SENSOR_DEVICE, SENSOR_POLL_TIME)?;
        let sensor =
            sensor::mgr_find_next_bydevname(&SENSOR_DEVICE,
                                            core::ptr::null_mut())?;
        if !!sensor.is_null() {
            {
                ::core::panicking::panic(&("no sensor",
                                           "rust/app/src/app_sensor.rs",
                                           56u32, 5u32))
            }
        };
        let listener =
            sensor_listener{sl_sensor_type: TEMP_SENSOR_TYPE,
                            sl_func:
                                sensor::as_untyped(handle_sensor_data),
                                                                          ..unsafe
                                                                            {
                                                                                ::core::mem::transmute::<[u8; ::core::mem::size_of::<sensor_listener>()],
                                                                                                         sensor_listener>([0;
                                                                                                                              ::core::mem::size_of::<sensor_listener>()])
                                                                            }};
        sensor::register_listener(sensor, listener)?;
        Ok(())
    }
    ///  This listener function is called every 10 seconds by Mynewt to handle the polled sensor data.
    ///  Return 0 if we have handled the sensor data successfully.
    extern "C" fn handle_sensor_data(sensor: sensor_ptr, _arg: sensor_arg,
                                     sensor_data: sensor_data_ptr,
                                     sensor_type: sensor_type_t)
     -> MynewtError {
        console::print("Rust handle_sensor_data\n");
        if sensor_data.is_null() { return MynewtError::SYS_EINVAL; }
        if !!sensor.is_null() {
            {
                ::core::panicking::panic(&("null sensor",
                                           "rust/app/src/app_sensor.rs",
                                           80u32, 5u32))
            }
        };
        let sensor_value = convert_sensor_data(sensor_data, sensor_type);
        if let SensorValueType::None = sensor_value.val {
            if !false {
                {
                    ::core::panicking::panic(&("bad type",
                                               "rust/app/src/app_sensor.rs",
                                               84u32, 55u32))
                }
            };
        }
        let res = send_sensor_data(&sensor_value);
        if let Err(err) = res {
            if err == MynewtError::SYS_EAGAIN {
                console::print("TMP network not ready\n");
                return MynewtError::SYS_EOK;
            }
        }
        MynewtError::SYS_EOK
    }
    ///  Convert the raw temperature value received from Mynewt into a `SensorValue` for transmission, which includes the sensor data key `t`. 
    ///  `sensor_type` indicates the type of data in `sensor_data`.
    #[allow(non_snake_case, unused_variables)]
    fn convert_sensor_data(sensor_data: sensor_data_ptr,
                           sensor_type: sensor_type_t) -> SensorValue {
        console::print("TMP listener got rawtmp\n");
        SensorValue{key: TEMP_SENSOR_KEY,
                    val:
                        match sensor_type {
                            SENSOR_TYPE_AMBIENT_TEMPERATURE_RAW => {
                                let mut rawtempdata =
                                    unsafe {
                                        ::core::mem::transmute::<[u8; ::core::mem::size_of::<sensor_temp_raw_data>()],
                                                                 sensor_temp_raw_data>([0;
                                                                                           ::core::mem::size_of::<sensor_temp_raw_data>()])
                                    };
                                let rc =
                                    unsafe {
                                        sensor::get_temp_raw_data(sensor_data,
                                                                  &mut rawtempdata)
                                    };
                                {
                                    match (&(rc), &(0)) {
                                        (left_val, right_val) => {
                                            if !(*left_val == *right_val) {
                                                {
                                                    ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                  "`,\n right: `",
                                                                                                                  "`: "],
                                                                                                                &match (&&*left_val,
                                                                                                                        &&*right_val,
                                                                                                                        &::core::fmt::Arguments::new_v1(&["rawtmp fail"],
                                                                                                                                                        &match ()
                                                                                                                                                             {
                                                                                                                                                             ()
                                                                                                                                                             =>
                                                                                                                                                             [],
                                                                                                                                                         }))
                                                                                                                     {
                                                                                                                     (arg0,
                                                                                                                      arg1,
                                                                                                                      arg2)
                                                                                                                     =>
                                                                                                                     [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                                   ::core::fmt::Debug::fmt),
                                                                                                                      ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                                                   ::core::fmt::Debug::fmt),
                                                                                                                      ::core::fmt::ArgumentV1::new(arg2,
                                                                                                                                                   ::core::fmt::Display::fmt)],
                                                                                                                 }),
                                                                                 &("rust/app/src/app_sensor.rs",
                                                                                   117u32,
                                                                                   17u32))
                                                }
                                            }
                                        }
                                    }
                                };
                                {
                                    match (&(rawtempdata.strd_temp_raw_is_valid),
                                           &(0)) {
                                        (left_val, right_val) => {
                                            if *left_val == *right_val {
                                                {
                                                    ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(&["assertion failed: `(left != right)`\n  left: `",
                                                                                                                  "`,\n right: `",
                                                                                                                  "`: "],
                                                                                                                &match (&&*left_val,
                                                                                                                        &&*right_val,
                                                                                                                        &::core::fmt::Arguments::new_v1(&["bad rawtmp"],
                                                                                                                                                        &match ()
                                                                                                                                                             {
                                                                                                                                                             ()
                                                                                                                                                             =>
                                                                                                                                                             [],
                                                                                                                                                         }))
                                                                                                                     {
                                                                                                                     (arg0,
                                                                                                                      arg1,
                                                                                                                      arg2)
                                                                                                                     =>
                                                                                                                     [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                                   ::core::fmt::Debug::fmt),
                                                                                                                      ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                                                   ::core::fmt::Debug::fmt),
                                                                                                                      ::core::fmt::ArgumentV1::new(arg2,
                                                                                                                                                   ::core::fmt::Display::fmt)],
                                                                                                                 }),
                                                                                 &("rust/app/src/app_sensor.rs",
                                                                                   119u32,
                                                                                   17u32))
                                                }
                                            }
                                        }
                                    }
                                };
                                SensorValueType::Uint(rawtempdata.strd_temp_raw)
                            }
                        },}
    }
}
mod app_network {
    //!  Transmit sensor data to a CoAP server like thethings.io.  The CoAP payload will be encoded as JSON.
    //!  The sensor data will be transmitted over NB-IoT.
    //!  Note that we are using a patched version of apps/my_sensor_app/src/vsscanf.c that
    //!  fixes response parsing bugs.  The patched file must be present in that location.
    //!  This is the Rust version of `https://github.com/lupyuen/stm32bluepill-mynewt-sensor/blob/rust-nbiot/apps/my_sensor_app/OLDsrc/network.c`
    use mynewt::{result::*, hw::sensor::{SensorValue, SensorValueType},
                 sys::console, encoding::coap_context::*,
                 libs::{sensor_network}, coap, d, Strn};
    use mynewt_macros::strn;
    /// Compose a CoAP JSON message with the Sensor Key (field name) and Value in `val`
    /// and send to the CoAP server.  The message will be enqueued for transmission by the CoAP / OIC 
    /// Background Task so this function will return without waiting for the message to be transmitted.
    /// Return `Ok()` if successful, `SYS_EAGAIN` if network is not ready yet.
    /// For the CoAP server hosted at thethings.io, the CoAP payload should be encoded in JSON like this:
    /// ```json
    /// {"values":[
    ///   {"key":"device", "value":"0102030405060708090a0b0c0d0e0f10"},
    ///   {"key":"t",      "value":1715}
    /// ]}
    /// ```
    pub fn send_sensor_data(val: &SensorValue) -> MynewtResult<()> {
        console::print("Rust send_sensor_data\n");
        let device_id = sensor_network::get_device_id()?;
        let rc = sensor_network::init_server_post(&Strn::new(b"\0"))?;
        if !rc { return Err(MynewtError::SYS_EAGAIN); }
        let _payload =
            {
                "begin json root";
                {
                    "begin json coap_root";
                    unsafe {
                        mynewt::libs::sensor_network::prepare_post(mynewt::encoding::APPLICATION_JSON)?;
                    }
                    unsafe {
                        mynewt::libs::sensor_coap::json_rep_start_root_object();
                    }
                    {
                        {
                            "begin json coap_array , object : COAP_CONTEXT , key : values";
                            {
                                "<< jarri , o: COAP_CONTEXT, k: values";
                                let key_with_null: &str = "values\u{0}";
                                unsafe {
                                    mynewt::libs::mynewt_rust::json_helper_set_array(COAP_CONTEXT.to_void_ptr(),
                                                                                     COAP_CONTEXT.key_to_cstr(key_with_null.as_bytes()));
                                };
                            };
                            {
                                " >>  >> \"device\" >> : & device_id , val ,";
                                "add1 key : \"device\" value : $crate::parse!(@ json &device_id) to object :\nCOAP_CONTEXT";
                                {
                                    "begin json coap_item_str , parent : COAP_CONTEXT , key : \"device\" , val :\n$crate::parse!(@ json &device_id)";
                                    {
                                        "begin json coap_item , array : COAP_CONTEXT";
                                        {
                                            "<< jitmi c: COAP_CONTEXT";
                                            let key_with_null: &str =
                                                "COAP_CONTEXT\u{0}";
                                            unsafe {
                                                mynewt::libs::mynewt_rust::json_helper_object_array_start_item(COAP_CONTEXT.key_to_cstr(key_with_null.as_bytes()))
                                            };
                                        };
                                        {
                                            {
                                                "-- jtxti o: COAP_CONTEXT, k: key, v: \"device\"";
                                                let key_strn: &Strn =
                                                    &Strn::new(b"key\x00");
                                                let value_strn: &Strn =
                                                    &Strn::new(b"device\x00");
                                                unsafe {
                                                    COAP_CONTEXT.json_set_text_string(key_strn,
                                                                                      value_strn)
                                                };
                                            };
                                            {
                                                "-- jtxti o: COAP_CONTEXT, k: value, v: $crate::parse!(@ json &device_id)";
                                                let key_strn: &Strn =
                                                    &Strn::new(b"value\x00");
                                                let value_strn: &Strn =
                                                    &device_id;
                                                unsafe {
                                                    COAP_CONTEXT.json_set_text_string(key_strn,
                                                                                      value_strn)
                                                };
                                            };
                                        };
                                        {
                                            ">>";
                                            let key_with_null: &str =
                                                "COAP_CONTEXT\u{0}";
                                            unsafe {
                                                mynewt::libs::mynewt_rust::json_helper_object_array_end_item(COAP_CONTEXT.key_to_cstr(key_with_null.as_bytes()))
                                            };
                                        };
                                        "end json coap_item";
                                    };
                                    "end json coap_item_str";
                                };
                                "--------------------";
                                " >>  >> val >> ,";
                                "--------------------";
                                {
                                    "begin json coap_item_int_val , c : COAP_CONTEXT , val : val";
                                    if let SensorValueType::Uint(val) =
                                           val.val {
                                        {
                                            "begin json coap_item_int , key : val.key , value : val";
                                            {
                                                "begin json coap_item , array : COAP_CONTEXT";
                                                {
                                                    "<< jitmi c: COAP_CONTEXT";
                                                    let key_with_null: &str =
                                                        "COAP_CONTEXT\u{0}";
                                                    unsafe {
                                                        mynewt::libs::mynewt_rust::json_helper_object_array_start_item(COAP_CONTEXT.key_to_cstr(key_with_null.as_bytes()))
                                                    };
                                                };
                                                {
                                                    {
                                                        "-- jtxte o: COAP_CONTEXT, k: \"key\", v: val.key";
                                                        let key_with_opt_null:
                                                                &[u8] =
                                                            "key".to_bytes_optional_nul();
                                                        let value_with_opt_null:
                                                                &[u8] =
                                                            val.key.to_bytes_optional_nul();
                                                        unsafe {
                                                            mynewt::libs::mynewt_rust::json_helper_set_text_string(COAP_CONTEXT.to_void_ptr(),
                                                                                                                   COAP_CONTEXT.key_to_cstr(key_with_opt_null),
                                                                                                                   COAP_CONTEXT.value_to_cstr(value_with_opt_null))
                                                        };
                                                    };
                                                    {
                                                        "-- jinte o: COAP_CONTEXT, k: \"value\", v: val";
                                                        let key_with_opt_null:
                                                                &[u8] =
                                                            "value".to_bytes_optional_nul();
                                                        let value =
                                                            val as u64;
                                                        unsafe {
                                                            mynewt::libs::mynewt_rust::json_helper_set_int(COAP_CONTEXT.to_void_ptr(),
                                                                                                           COAP_CONTEXT.key_to_cstr(key_with_opt_null),
                                                                                                           value)
                                                        };
                                                    };
                                                };
                                                {
                                                    ">>";
                                                    let key_with_null: &str =
                                                        "COAP_CONTEXT\u{0}";
                                                    unsafe {
                                                        mynewt::libs::mynewt_rust::json_helper_object_array_end_item(COAP_CONTEXT.key_to_cstr(key_with_null.as_bytes()))
                                                    };
                                                };
                                                "end json coap_item";
                                            };
                                            "end json coap_item_int";
                                        };
                                    } else {
                                        unsafe {
                                            COAP_CONTEXT.fail(CoapError::VALUE_NOT_UINT)
                                        };
                                    }
                                    "end json coap_item_int_val";
                                };
                                "--------------------";
                            };
                            {
                                ">>";
                                let key_with_null: &str = "values\u{0}";
                                unsafe {
                                    mynewt::libs::mynewt_rust::json_helper_close_array(COAP_CONTEXT.to_void_ptr(),
                                                                                       COAP_CONTEXT.key_to_cstr(key_with_null.as_bytes()))
                                };
                            };
                            "end json coap_array";
                        };
                    };
                    unsafe {
                        mynewt::libs::sensor_coap::json_rep_end_root_object();
                    }
                    "end json coap_root";
                };
                "end json root";
                ()
            };
        sensor_network::do_server_post()?;
        console::print("NET view your sensor at \nhttps://blue-pill-geolocate.appspot.com?device=");
        console::print_strn(&device_id);
        console::print("\n");
        Ok(())
    }
}
use core::panic::PanicInfo;
use cortex_m::asm::bkpt;
use mynewt::{kernel::os, sys::console, libs::sensor_network};
///  Main program that initialises the sensor, network driver and starts reading and sending sensor data in the background.
///  main() will be called at Mynewt startup. It replaces the C version of the main() function.
#[no_mangle]
extern "C" fn main() -> ! {
    mynewt::sysinit();
    app_sensor::start_sensor_listener().expect("TMP fail");
    sensor_network::start_server_transport().expect("NET fail");
    loop  {
        os::eventq_run(os::eventq_dflt_get().expect("GET fail")).expect("RUN fail");
    }
}
///  This function is called on panic, like an assertion failure. We display the filename and line number and pause in the debugger. From https://os.phil-opp.com/freestanding-rust-binary/
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    console::print("panic ");
    if let Some(location) = info.location() {
        let file = location.file();
        let line = location.line();
        console::print("at ");
        console::buffer(&file);
        console::print(" line 0x");
        console::printhex(line as u8);
        console::print("\n");
        console::flush();
    } else { console::print("no loc\n"); console::flush(); }
    bkpt();
    loop  { }
}
