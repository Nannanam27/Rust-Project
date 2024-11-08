use iced::widget::{button, column, text, Container};
use iced::{Element, Sandbox, Settings, Theme};
use sysinfo::{System, SystemExt, CpuExt, DiskExt, NetworkExt};
use std::fs::OpenOptions;
use std::io::{Error, Write};
use chrono::{Local, Timelike};

pub fn main() -> iced::Result {
    Task::run(Settings::default())
}

#[derive(Default)]
enum Task {
    #[default]
    Loading,
    Loaded {
        information: SystemInformation,
        show_cpu_usage: bool, 
    },
}

struct SystemInformation {
    cpu_usages: Vec<f32>,
    used_memory: u64,
    total_memory: u64,
    used_swap: u64,
    total_swap: u64,
    disks: Vec<(String, u64, u64)>,
    networks: Vec<(String, u64, u64)>,
}

#[derive(Clone, Debug)]
enum Message {
    Refresh,
    CpuUsage, 
}

impl Sandbox for Task {
    type Message = Message;

    fn new() -> Self {
        let mut app = Task::Loading;
        app.update(Message::Refresh);
        app
    }

    fn title(&self) -> String {
        String::from("System Monitor - Rust Project")
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Refresh => {
                let mut sys = System::new_all();
                sys.refresh_all();

                let mut network_info = Vec::new();
                for (name, data) in sys.networks() {
                    network_info.push((name.clone(), data.received(), data.transmitted()));
                }

                let information = SystemInformation {
                    cpu_usages: sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect(),
                    used_memory: sys.used_memory(),
                    total_memory: sys.total_memory(),
                    used_swap: sys.used_swap(),
                    total_swap: sys.total_swap(),
                    disks: sys.disks().iter().map(|disk| (
                        disk.name().to_string_lossy().into_owned(),
                        disk.total_space(),
                        disk.total_space() - disk.available_space(),
                    )).collect(),
                    networks: network_info,
                };


                if let Err(err) = file("system_info.txt", &information) {
                    println!("Error writing to file: {}", err);
                }

                *self = Self::Loaded {
                    information,
                    show_cpu_usage: false, 
                };
            }
            Message::CpuUsage => {
                if let Task::Loaded { ref mut show_cpu_usage, .. } = *self {
                    *show_cpu_usage = !*show_cpu_usage; 
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content: Element<_> = match self {
            Task::Loading => text("Loading...").size(40).into(),
            Task::Loaded { information, show_cpu_usage } => {
                let memory_per = (information.used_memory as f64 / information.total_memory as f64) * 100.0;
                let swap_per = (information.used_swap as f64 / information.total_swap as f64) * 100.0;

                let mut column_content = column![
                    text("                        System Monitor").size(30),
                    text(format!(
                        "   Memory: used {:.2} TB / total {:.2} TB ({:.2}%)",
                        information.used_memory as f64 / 1_048_576.0,
                        information.total_memory as f64 / 1_048_576.0,
                        memory_per
                    )),
                    text(format!(
                        "   Swap: used {:.2} TB / total {:.2} TB ({:.2}%)",
                        information.used_swap as f64 / 1_048_576.0,
                        information.total_swap as f64 / 1_048_576.0,
                        swap_per
                    )),
 
                ]
                .spacing(10);

                column_content = column_content.push(
                    button("    CPU Usage")
                        .on_press(Message::CpuUsage)
                );

                if *show_cpu_usage {
                    for (i, usage) in information.cpu_usages.iter().enumerate() {
                        column_content = column_content.push(
                            text(format!("          CPU {}: {:.2}%", i + 1, usage)).size(20)
                        );
                    }
                }

                column_content = column_content.push(text("    Disk usage:").size(20));
                for (name, total, used) in &information.disks {
                    let disk_usage_percentage = (*used as f64 / *total as f64) * 100.0;
                    column_content = column_content.push(text(format!(
                        "           {}: {:.2} GB used / {:.2} GB total ({:.2}%)",
                        name,
                        *used as f64 / 1_073_741_824.0,
                        *total as f64 / 1_073_741_824.0,
                        disk_usage_percentage
                    )));
                }
                

                column_content = column_content.push(text("    Network usage:").size(20));
                for (name, received, transmitted) in &information.networks {
                    column_content = column_content.push(text(format!(
                        "           {}: received {} KB / transmitted {} KB",
                        name,
                        *received / 1024,
                        *transmitted / 1024
                    )));
                }

                column_content = column_content.push(button("Refresh").on_press(Message::Refresh));

                column_content.into()
            }
        };

        Container::new(content).center_x().center_y().width(iced::Length::Fill).height(iced::Length::Fill).into()
    }
}

fn file(path: &str, information: &SystemInformation) -> Result<(), Error> {
    let mut file = OpenOptions::new().write(true).create(true).append(true).open(path)?;
    let memory_usage_percentage = (information.used_memory as f64 / information.total_memory as f64) * 100.0;
    let swap_usage_percentage = (information.used_swap as f64 / information.total_swap as f64) * 100.0;

    let now = Local::now();
    let formatted_time = format!("{:02}:{:02}", now.hour(), now.minute());

    let cpu_usage = information.cpu_usages
        .iter().enumerate().map(|(i, usage)| format!("CPU {}: {:.2}%", i + 1, usage)).collect::<Vec<String>>()
        .join(", ");

    let disk_usage = information.disks
    .iter()
    .map(|(name, total, used)| {
        let disk_usage_percentage = (*used as f64 / *total as f64) * 100.0;
        format!(
            "{}: {:.2} GB used / {:.2} GB total ({:.2}%)",
            name,
            *used as f64 / 1_073_741_824.0,
            *total as f64 / 1_073_741_824.0,
            disk_usage_percentage
        )
    }).collect::<Vec<String>>().join(", ");

    let network_usage = information.networks
        .iter().map(|(name, received, transmitted)| {
            format!(
                "{}: received {} KB / transmitted {} KB",
                name,
                *received / 1024,
                *transmitted / 1024
            )
        }).collect::<Vec<String>>().join(", ");

    let data = format!(
        "Time: {}\nMemory: used {:.2} TB / total {:.2} TB ({:.2}%)\nSwap: used {:.2} TB / total {:.2} TB ({:.2}%)\nCPU Usage: {}\nDisk Usage: {}\nNetwork Usage: {}\n\n",
        formatted_time,
        information.used_memory as f64 / 1_048_576.0,
        information.total_memory as f64 / 1_048_576.0,
        memory_usage_percentage,
        information.used_swap as f64 / 1_048_576.0,
        information.total_swap as f64 / 1_048_576.0,
        swap_usage_percentage,
        cpu_usage,
        disk_usage,
        network_usage
    );

    file.write_all(data.as_bytes())?;
    Ok(())
}