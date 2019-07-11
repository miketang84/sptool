extern crate clap;
extern crate serialport;
use serde::{Serialize, Deserialize};

use std::io::{self, Write};
use std::time::Duration;

use clap::{App, AppSettings, Arg};
use serialport::prelude::*;

// {"MSGID":"12345678","TYPE":"WRITE_DEVICE","DEVICE_ID":"10000001","IC_ID":"12345678","MSG":"HAVE_CARD","TIME":""}
// {"MSGID":"12345678","TYPE":"WRITE_DEVICE","DEVICE_ID":"10000001","IC_ID":"10001111","MSG":"WRITE","TIME":""}
// {"MSGID":"0","TYPE":"4G_GPS_DEVICE","DEVICE_ID":"10000001","IC_ID":"","TIME":"0","MSG":"HEART","LON":"12.345678","LAT":"22.222222"}
// {"MSGID":"0","TYPE":"4G_GPS_DEVICE","DEVICE_ID":"10000001","IC_ID":"","TIME":"68000012","MSG":"HEART_OKD"}
// {"MSGID":"12345678","TYPE":"4G_GPS_DEVICE","DEVICE_ID":"10000001","IC_ID":"10000001-68000123","TIME":"68000012","MSG":"UPDATA"}
// {"MSGID":"12345678","TYPE":"4G_GPS_DEVICE","DEVICE_ID":"10000001","IC_ID":"","TIME":"68000012","MSG":"UPDATA_OK"}
#[derive(Deserialize, Serialize, Debug)]
struct Msg {
    MSGID: String,
    TYPE: String,
    DEVICE_ID: String,
    IC_ID: String,
    TIME: String,
    MSG: String,
}


fn main() {
    let matches = App::new("Serialport Example - Receive Data")
        .about("Reads data from a serial port and echoes it to stdout")
        .setting(AppSettings::DisableVersion)
        .arg(
            Arg::with_name("port")
                .help("The device path to a serial port")
                .use_delimiter(false)
                .required(true),
        )
        .arg(
            Arg::with_name("baud")
                .help("The baud rate to connect at")
                .use_delimiter(false)
                .required(true),
        )
        .get_matches();
    let port_name = matches.value_of("port").unwrap();
    let baud_rate = matches.value_of("baud").unwrap();

    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(10);
    if let Ok(rate) = baud_rate.parse::<u32>() {
        settings.baud_rate = rate.into();
    } else {
        eprintln!("Error: Invalid baud rate '{}' specified", baud_rate);
        ::std::process::exit(1);
    }

    match serialport::open_with_settings(&port_name, &settings) {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            println!("==== Listening on {} at {} baud ====", &port_name, &baud_rate);
            let mut text_string = String::new();
            loop {
                //match port.read(serial_buf.as_mut_slice()) {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(t) => {
                        //io::stdout().write_all(&serial_buf[..t]).unwrap();
                        //io::stdout().flush().unwrap();
                        //println!("{:?}", serial_buf);
                        //println!("{:?}", t);
                        let json_text = String::from_utf8_lossy(&serial_buf[..t]);
                        //println!("{:?}", json_text);
                        text_string.push_str(&json_text);
                        //println!("{:?}", text_string);
                        let p: Result<Msg, _> = serde_json::from_str(&text_string);
                        match p {
                            Ok(p) => {
                                // 这里p是解析出来的json数据，下面用它做业务处理
                                //println!("{:?}", p);
                                if &p.MSG == "HAVE_CARD" {
                                    // 有新卡来了，需要写卡
                                    println!("Received HAVE_CARD message: {}", &text_string);
                                    let mut new_id = String::new();
                                    print!("Please input new card ID: ");
                                    std::io::stdout().flush().unwrap();

                                    io::stdin().read_line(&mut new_id)
                                        .expect("Failed to read line");

                                    let write_msg = Msg {
                                        MSGID: p.MSGID.to_string(),
                                        TYPE: p.TYPE.to_string(),
                                        DEVICE_ID: p.DEVICE_ID.to_string(),
                                        IC_ID: new_id.replace("\r\n", "\n").replace("\n", "").to_string(),  // 这里由上层业务产生一个新卡号
                                        TIME: time::now().to_timespec().sec.to_string(),
                                        MSG: "WRITE".to_string()
                                    };
                                    let j = serde_json::to_string(&write_msg).unwrap();
                                    // {"MSGID":"12345678","TYPE":"WRITE_DEVICE","DEVICE_ID":"10000001","IC_ID":"10001111","MSG":"WRITE"}
                                    match port.write(j.as_bytes()) {
                                        Ok(_) => {
                                            println!("Send: {}", j);
                                            //std::io::stdout().flush().unwrap();
                                        }
                                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                                        Err(e) => eprintln!("{:?}", e),
                                    }

                                }
                                else if &p.MSG == "WRITE_OK" {
                                    // 写卡成功的回执

                                }
                                else if &p.MSG == "HEART" {
                                    println!("{}", "in heartbeat");
                                    // 如果是心跳包
                                    if &p.TIME == "0" {
                                        // 获取当前系统时间，并用于产生新的msg
                                        let msg = Msg {
                                            MSGID: p.MSGID.to_string(),
                                            TYPE: p.TYPE.to_string(),
                                            DEVICE_ID: p.DEVICE_ID.to_string(),
                                            IC_ID: p.IC_ID.to_string(),
                                            TIME: time::now().to_timespec().sec.to_string(),
                                            MSG: "HEART_OKD".to_string()
                                        };
                                        let j = serde_json::to_string(&msg).unwrap();

                                        match port.write(j.as_bytes()) {
                                            Ok(_) => {
                                                println!("Send: {}", j);
                                                //std::io::stdout().flush().unwrap();
                                            }
                                            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                                            Err(e) => eprintln!("{:?}", e),
                                        }

                                    }
                                }
                                else if &p.MSG == "UPDATA" {
                                    // 这里处理读卡后数据上传逻辑



                                    // 处理完后，返回 UPDATA_OK
                                    let msg = Msg {
                                        MSGID: p.MSGID.to_string(),
                                        TYPE: p.TYPE.to_string(),
                                        DEVICE_ID: p.DEVICE_ID.to_string(),
                                        IC_ID: p.IC_ID.to_string(),
                                        TIME: time::now().to_timespec().sec.to_string(),
                                        MSG: "UPDATA_OK".to_string()
                                    };
                                    let j = serde_json::to_string(&msg).unwrap();

                                    match port.write(j.as_bytes()) {
                                        Ok(_) => {
                                            println!("Send: {}", j);
                                            //std::io::stdout().flush().unwrap();
                                        }
                                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                                        Err(e) => eprintln!("{:?}", e),
                                    }

                                }

                                // 最后要把这个字符串缓冲清除
                                text_string = String::new();
                            },
                            Err(_) => {

                            }
                        }

                        
                        // println!("{:?}", p);

                    },
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }
}
