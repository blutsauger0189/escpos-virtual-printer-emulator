use crate::emulator::EmulatorState;
use egui::Ui;
use std::net::TcpStream;
use std::process::Command;
use std::time::Duration;

pub struct SettingsPanel {
    // No more useless settings - the emulator works according to ESC/POS standards
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self {}
    }
}

impl SettingsPanel {
    fn current_os() -> &'static str {
        std::env::consts::OS
    }

    fn printer_name() -> &'static str {
        match Self::current_os() {
            "windows" => "ESC_POS_Virtual_Printer",
            "linux" => "ESC_POS_Linux_Printer",
            "macos" => "ESC_POS_macOS_Printer",
            _ => "ESC_POS_Virtual_Printer",
        }
    }

    pub fn show(&mut self, ui: &mut Ui, _state: &mut EmulatorState) {
        ui.heading("Emulator Settings");
        ui.separator();

        // Virtual printer management
        ui.group(|ui| {
            ui.label("Virtual Printer Management");
            ui.label("Installs the emulator as a system printer");

            ui.horizontal(|ui| {
                match Self::current_os() {
                    "windows" => {
                        if ui.button("🖨️ Install Windows Printer").clicked() {
                            self.install_windows_printer();
                        }
                    }
                    "linux" => {
                        if ui.button("🐧 Install Linux Printer").clicked() {
                            self.install_linux_printer();
                        }
                    }
                    "macos" => {
                        if ui.button("🍎 Install macOS Printer").clicked() {
                            self.install_macos_printer();
                        }
                    }
                    other => {
                        ui.label(format!("Printer installation not supported on {}", other));
                    }
                }

                if ui.button("🗑️ Uninstall Printer").clicked() {
                    self.uninstall_printer();
                }
            });

            ui.label("Note: Requires administrator privileges");

            // Check printer status
            if ui.button("🔍 Check Status").clicked() {
                self.check_printer_status();
            }
        });

        ui.separator();

        // Network settings
        ui.group(|ui| {
            ui.label("Network Configuration");
            ui.label("TCP Port: 9100");
            ui.label("Address: 127.0.0.1");

            if ui.button("📡 Test Connection").clicked() {
                self.test_network_connection();
            }
        });

        ui.separator();

        // Information about operation
        ui.group(|ui| {
            ui.label("ℹ️  Automatic Operation");
            ui.label("• The emulator automatically respects ESC/POS standards");
            ui.label("• Paper width: 50mm, 78mm, 80mm (auto-detection)");
            ui.label("• Font, justification, emphasis: ESC/POS commands");
            ui.label("• No manual configuration needed!");
        });
    }

    fn install_windows_printer(&self) {
        let name = Self::printer_name();
        let script = format!(
            "Add-PrinterPort -Name '127.0.0.1:9100' -PrinterHostAddress '127.0.0.1' -PortNumber 9100; \
             $driver = (Get-PrinterDriver | Where-Object {{ $_.Name -like '*Microsoft*' }} | Select-Object -First 1).Name; \
             Add-Printer -Name '{name}' -DriverName $driver -PortName '127.0.0.1:9100'; \
             Write-Host 'Printer installed successfully'"
        );

        let output = Command::new("powershell")
            .args(["-Command", &script])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    println!("✅ {}", stdout);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("❌ Error: {}", stderr);
                }
            }
            Err(e) => {
                println!("❌ Cannot execute printer installation: {}", e);
            }
        }
    }

    fn install_linux_printer(&self) {
        let name = Self::printer_name();
        let script = format!(
            "if command -v lpstat &> /dev/null; then \
                echo 'Installing Linux printer...'; \
                sudo lpadmin -p {name} -E -v socket://127.0.0.1:9100 -m 'Generic Text-Only Printer'; \
                sudo lpadmin -d {name}; \
                echo 'Linux printer installed successfully!'; \
            else \
                echo 'CUPS not found. Please install CUPS first.'; \
            fi"
        );

        let output = Command::new("bash").args(["-c", &script]).output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    println!("ℹ️  {}", stdout);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("ℹ️  {}", stderr);
                }
            }
            Err(e) => {
                println!("ℹ️  Linux installation attempted: {}", e);
            }
        }
    }

    fn install_macos_printer(&self) {
        let name = Self::printer_name();
        let shell_cmd = format!(
            "lpadmin -p {name} -E -v socket://127.0.0.1:9100 -m drv:///sample.drv/generic.ppd && \
             cupsenable {name} && cupsaccept {name}"
        );
        self.run_macos_privileged(&shell_cmd, "macOS printer installed successfully");
    }

    /// Run a shell command with admin privileges on macOS using the native
    /// Authorization Services prompt (the standard system password dialog)
    /// instead of `sudo`, which would require a TTY and hang when the app is
    /// launched from Finder.
    fn run_macos_privileged(&self, shell_cmd: &str, success_msg: &str) {
        let escaped = shell_cmd.replace('\\', "\\\\").replace('"', "\\\"");
        let applescript = format!(
            "do shell script \"{}\" with administrator privileges",
            escaped
        );

        let output = Command::new("osascript").args(["-e", &applescript]).output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("ℹ️  {}", success_msg);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if !stdout.trim().is_empty() {
                        println!("{}", stdout);
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("❌ {}", stderr.trim());
                }
            }
            Err(e) => {
                println!("❌ Cannot launch osascript: {}", e);
            }
        }
    }

    fn uninstall_printer(&self) {
        let name = Self::printer_name();

        let output = match Self::current_os() {
            "windows" => {
                let script = format!(
                    "Remove-Printer -Name '{name}' -Confirm:$false; \
                     Remove-PrinterPort -Name '127.0.0.1:9100'; \
                     Write-Host 'Printer uninstalled successfully'"
                );
                Command::new("powershell")
                    .args(["-Command", &script])
                    .output()
            }
            "macos" => {
                self.run_macos_privileged(
                    &format!("lpadmin -x {name}"),
                    "Printer uninstalled successfully",
                );
                return;
            }
            "linux" => {
                let script = format!(
                    "if command -v lpadmin &> /dev/null; then \
                        sudo lpadmin -x {name} && echo 'Printer uninstalled successfully'; \
                    else \
                        echo 'CUPS not found. lpadmin is required.'; \
                    fi"
                );
                Command::new("bash").args(["-c", &script]).output()
            }
            other => {
                println!("❌ Uninstall not supported on {}", other);
                return;
            }
        };

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    println!("✅ {}", stdout);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("❌ Error: {}", stderr);
                }
            }
            Err(e) => {
                println!("❌ Cannot execute printer uninstallation: {}", e);
            }
        }
    }

    fn check_printer_status(&self) {
        let name = Self::printer_name();

        let output = match Self::current_os() {
            "windows" => {
                let script = format!(
                    "Get-Printer -Name '{name}' -ErrorAction SilentlyContinue | \
                     Select-Object Name, PortName, DriverName, PrinterStatus"
                );
                Command::new("powershell")
                    .args(["-Command", &script])
                    .output()
            }
            "linux" | "macos" => Command::new("lpstat").args(["-p", name]).output(),
            other => {
                println!("❌ Status check not supported on {}", other);
                return;
            }
        };

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if output.status.success() && !stdout.trim().is_empty() {
                    println!("✅ Virtual printer installed:");
                    println!("{}", stdout);
                } else {
                    println!("ℹ️  Virtual printer not installed");
                }
            }
            Err(e) => {
                println!("❌ Cannot check status: {}", e);
            }
        }
    }

    fn test_network_connection(&self) {
        let address = "127.0.0.1:9100".parse().unwrap();
        match TcpStream::connect_timeout(&address, Duration::from_secs(2)) {
            Ok(_) => println!("✅ Connection to port 9100 successful"),
            Err(e) => println!("❌ Connection to port 9100 failed: {}", e),
        }
    }
}
