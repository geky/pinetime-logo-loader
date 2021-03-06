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

//  Mynewt Driver for Remote Sensor.  The Remote Sensor driver is installed in the Collector Node.
//  It receives sensor data from Sensor Nodes via nRF24L01 encoded with CBOR.  
//  We automatically create a Remote Sensor device for each Sensor Node.
//  When a message is received from a Sensor Node, we trigger the Listener Function
//  for the associated Remote Sensor device.
//  A Remote Sensor behaves like a local sensor, so Mynewt sensor listeners will work.
//  With Remote Sensor we may build a sensor data router on the Collector Node that 
//  receives sensor data from Sensor Nodes and transmits to a CoAP Server.

#ifndef __REMOTE_SENSOR_H__
#define __REMOTE_SENSOR_H__

#include "os/mynewt.h"
#include "sensor/sensor.h"

#ifdef __cplusplus
extern "C" {
#endif

//  Configuration for the Remote Sensor
struct remote_sensor_cfg {
    sensor_type_t bc_s_mask;   //  Sensor data types that will be returned, i.e. temperature.
    const char *addr;          //  Address of the sender node.
};

//  Device for the Remote Sensor
struct remote_sensor {
    struct os_dev dev;     //  Mynewt device
    struct sensor sensor;  //  Mynewt sensor
    struct remote_sensor_cfg cfg;  //  Sensor configuration
    os_time_t last_read_time;   //  Last time the sensor was read.
    struct os_eventq sensor_data_queue;  //  Received sensor data to be processed.
};

/**
 * Create the Remote Sensor instance.  Implemented in creator.c, function DEVICE_CREATE().
 */
void remote_sensor_create(void);

/**
 * Return the default configuration for the Remote Sensor.
 *
 * @param cfg  Pointer to the remote_sensor_cfg device config
 *
 * @return 0 on success, and non-zero error code on failure
 */
int remote_sensor_default_cfg(struct remote_sensor_cfg *cfg);

/**
 * Initialize the Remote Sensor.
 *
 * @param dev  Pointer to the remote_sensor_dev device descriptor
 *
 * @return 0 on success, and non-zero error code on failure
 */
int remote_sensor_init(struct os_dev *dev, void *arg);

/**
 * Configure Remote Sensor
 *
 * @param Sensor device remote_sensor structure
 * @param Sensor device remote_sensor_cfg config
 *
 * @return 0 on success, and non-zero error code on failure
 */
int remote_sensor_config(struct remote_sensor *remote_sensor, struct remote_sensor_cfg *cfg);

//  Return the Sensor Type given the CBOR field name.  Return 0 if not found.
sensor_type_t remote_sensor_lookup_type(const char *name);

//  Start the router that receives CBOR messages from Sensor Nodes
//  and triggers the Remote Sensor for the field names in the CBOR message. 
//  The router is started only for Collector Node.  Return 0 if successful.
int remote_sensor_start(void);

#ifdef __cplusplus
}
#endif

#endif /* __REMOTE_SENSOR_H__ */
