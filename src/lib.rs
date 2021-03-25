pub mod serial_port {
    use std::{io, time};

    const SERIAL_READ_BUFFER_SIZE: usize = 32;
    const SERIAL_OPEN_TIMEOUT_MS: u64 = 10;

    /// Represents a response given by `SerialPort` methods
    /// in order to make the implementation of the RPC easier.
    #[derive(Debug)]
    pub struct SerialPortResponse {
        pub success: bool,
        pub content: String,
    }

    /// Represents a POSIX or Windows serial port.
    pub struct SerialPort {
        /// There can be no real port associated to it.
        port: Option<Box<dyn serialport::SerialPort>>,
    }

    impl SerialPort {
        pub fn new() -> SerialPort {
            SerialPort { port: None }
        }

        /// Opens a serial port.
        ///
        /// # Paramters
        ///
        /// - `port_path`: The path to the serial port. Can be given by `get_available_port_names()`.
        /// - `baudrate`: The baudrate used to configure the serial communication.
        ///
        /// # Returns
        ///
        /// A `SerialPortResponse` containing:
        /// - `content`: informative message.
        /// - `success`: if the port has been open correctly.
        pub fn open_port(&mut self, port_path: &str, baudrate: u32) -> SerialPortResponse {
            if let Some(_port) = &self.port {
                return SerialPortResponse {
                    success: false,
                    content: "A port is already open".to_string(),
                };
            }

            // TODO check the input.

            let port_builder = serialport::new(port_path, baudrate)
                .timeout(time::Duration::from_millis(SERIAL_OPEN_TIMEOUT_MS));

            let port = port_builder.open();

            match port {
                Ok(port) => {
                    let port_path = match port.name() {
                        Some(name) => name,
                        None => "default".to_string(),
                    };

                    let baudrate = match port.baud_rate() {
                        Ok(baudrate) => baudrate,
                        Err(_) => 0,
                    };

                    self.port = Some(port);

                    return SerialPortResponse {
                        success: true,
                        content: format!(
                            "Openend port {} with a baudrate of {}",
                            port_path, baudrate
                        )
                        .to_string(),
                    };
                }
                Err(_e) => {
                    return SerialPortResponse {
                        success: false,
                        content: "Could not open the port".to_string(),
                    };
                }
            }
        }

        /// Closes the current serial port.
        ///
        /// # Paramters
        ///
        /// # Returns
        ///
        /// A `SerialPortResponse` containing:
        /// - `content`: informative message.
        /// - `success`: if the port has been closed correctly.
        pub fn close_port(&mut self) -> SerialPortResponse {
            if let Some(port) = self.port.as_mut() {
                let port_path = match port.name() {
                    Some(name) => name,
                    None => "default".to_string(),
                };

                drop(port);
                self.port = None;

                return SerialPortResponse {
                    success: true,
                    content: format!("Port {} closed", port_path).to_string(),
                };
            } else {
                return SerialPortResponse {
                    success: false,
                    content: "No port is currently open".to_string(),
                };
            }
        }

        /// Sends a message to the current opened serial port.
        ///
        /// # Paramters
        ///
        /// - `message`: The string slice to send.
        ///
        /// # Returns
        ///
        /// A `SerialPortResponse` containing:
        /// - `content`: informative message.
        /// - `success`: if the message has been sent correctly.
        pub fn send_once(&mut self, message: &str) -> SerialPortResponse {
            let output = parse_str_to_serial(message);
            let output = output.as_bytes();

            if let Some(port) = self.port.as_mut() {
                match port.write(output) {
                    Ok(_t) => {
                        return SerialPortResponse {
                            success: true,
                            content: "Request sent".to_string(),
                        };
                    }

                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                        return SerialPortResponse {
                            success: false,
                            content: "Serial write timed out".to_string(),
                        };
                    }
                    Err(e) => {
                        return SerialPortResponse {
                            success: false,
                            content: format!("Serial write error: {}", e).to_string(),
                        };
                    }
                }
            } else {
                return SerialPortResponse {
                    success: false,
                    content: "No port is currently open".to_string(),
                };
            }
        }

        /// Reads [TODO nb char] from the opened serial port.
        ///
        /// # Paramters
        ///
        /// # Returns
        ///
        /// A `SerialPortResponse` containing:
        /// - `content`: The characters read from the serial port, or an informative message.
        /// - `success`: if the chars has been correctly read from the serial port.
        pub fn read_once(&mut self) -> SerialPortResponse {
            if let Some(port) = self.port.as_mut() {
                let mut serial_buf: Vec<u8> = vec![0; SERIAL_READ_BUFFER_SIZE];

                match port.read(serial_buf.as_mut_slice()) {
                    Ok(t) => {
                        let content = String::from_utf8_lossy(&serial_buf[..t]).to_string();
                        println!("From serial: {}", content);

                        return SerialPortResponse {
                            success: true,
                            content: content,
                        };
                    }

                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                        return SerialPortResponse {
                            success: false,
                            content: "Serial read timed out".to_string(),
                        };
                    }
                    Err(e) => {
                        return SerialPortResponse {
                            success: false,
                            content: format!("Serial read error: {}", e).to_string(),
                        };
                    }
                }
            } else {
                return SerialPortResponse {
                    success: false,
                    content: "No port is currently open".to_string(),
                };
            }
        }

        /// Returns a list of available ports.
        pub fn get_available_port_names() -> Vec<String> {
            let ports = match serialport::available_ports() {
                Ok(ports) => ports,
                Err(_) => return vec!["No ports available".to_string()],
            };

            if ports.is_empty() {
                return vec!["No ports available".to_string()];
            }

            let mut port_names = vec![];

            for p in ports {
                port_names.push(p.port_name);
            }

            port_names
        }
    }

    use std::char;

    const CHAR_0_AS_U32: u32 = '0' as u32;
    const CHAR_9_AS_U32: u32 = '9' as u32;
    const CHAR_A_AS_U32: u32 = 'A' as u32;
    const CHAR_F_AS_U32: u32 = 'F' as u32;

    /// Mainly parses written/ascii hex value to real hex value (from 0x00 to 0xFF).
    // pub fn parse_str_to_serial(s: String) -> String {
    pub fn parse_str_to_serial(s: &str) -> String {
        let mut parsed_s = String::from("");

        if s.len() < 4 {
            return String::from(s);
        }

        let vec_s = s.chars().collect::<Vec<char>>();
        let mut hex_windows_it = vec_s.windows(4);
        let mut is_hex;
        let mut hex_int: u32;
        let mut hex_c: char = '0';

        // Looking for hex in the form 0xAA.
        while let Some(hex_word) = hex_windows_it.next() {
            is_hex = false;
            hex_int = 0;

            if hex_word[0] == '0' && (hex_word[1] == 'X' || hex_word[1] == 'x') {
                let hex_word_2 = hex_word[2] as u32;
                let hex_word_3 = hex_word[3] as u32;
                is_hex = true;

                if hex_word_2 >= CHAR_0_AS_U32 && hex_word_2 <= CHAR_9_AS_U32 {
                    hex_int += (hex_word_2 - CHAR_0_AS_U32) << 4;
                } else if hex_word_2 >= CHAR_A_AS_U32 && hex_word_2 <= CHAR_F_AS_U32 {
                    hex_int += (hex_word_2 - CHAR_A_AS_U32) << 4;
                } else {
                    is_hex = false;
                }

                if hex_word_3 >= CHAR_0_AS_U32 && hex_word_3 <= CHAR_9_AS_U32 {
                    hex_int += hex_word_3 - CHAR_0_AS_U32;
                } else if hex_word_3 >= CHAR_A_AS_U32 && hex_word_3 <= CHAR_F_AS_U32 {
                    hex_int += hex_word_3 - CHAR_A_AS_U32;
                } else {
                    is_hex = false;
                }

                if let Some(hex_int_to_char) = char::from_u32(hex_int) {
                    hex_c = hex_int_to_char;
                } else {
                    is_hex = false;
                }
            }

            if is_hex {
                parsed_s.push(hex_c);
                // Skips 3 next items.
                hex_windows_it.nth(2);
            } else {
                parsed_s.push(hex_word[0]);
            }
        }

        parsed_s
    }
}

#[cfg(test)]
mod tests {
    use super::serial_port::*;

    #[test]
    fn parse_str_untouched() {
        assert_eq!("ok", parse_str_to_serial("ok"));
    }

    #[test]
    fn parse_str_hex() {
        assert_eq!("\x02#", parse_str_to_serial("0x020x23"));
        assert_eq!(
            "\x02iii\x17ii\x03",
            parse_str_to_serial("0x02iii0x17ii0x03")
        );
    }
}
