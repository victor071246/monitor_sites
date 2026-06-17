use eframe::egui;
use tray_icon::{TrayIcon, TrayIconBuilder, menu::{Menu, MenuItem}};
use tray_icon::Icon;
use gtk;

pub struct AppGui {
    dominio: String,
    ip: String,
    portas: String,
    mostrar_formulario: bool,
    hosts: Vec<String>,
    porta_22: bool,
    porta_80: bool,
    porta_443: bool,
    porta_3306: bool,
    porta_5432: bool,
    porta_8080: bool,
    inicializado: bool,
}

impl Default for AppGui {
    fn default() -> Self{
        Self {
            dominio: String::new(),
            ip: String::new(),
            portas: String::new(),
            mostrar_formulario: false,
            hosts: Vec::new(),
            porta_22: false,
            porta_80: false,
            porta_443: false,
            porta_3306: false,
            porta_5432: false,
            porta_8080: false,
            inicializado: false,
        }
    }
}

pub fn iniciar() {
    gtk::init().expect("falha ao inicializar GTK");

    //carrega o icone - bytes embutios no binário em tempo de compilação
    let icone_bytes = include_bytes!("../../assets/icon.png");
    let imagem = image::load_from_memory(icone_bytes)
        .expect("erro ao carregar ícone")
        .into_rgba8();
    let (largura, altura) = imagem.dimensions();
    let icone = Icon::from_rgba(imagem.into_raw(), largura, altura)
        .expect("erro ao criar ícone");

    let menu = Menu::new();
    let item_sair = MenuItem::new("Sair", true, None);
    menu.append(&item_sair).ok();

    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Monitor Sites")
        .with_icon(icone)
        .build()
        .expect("erro ao criar tray icon");


    let opcoes = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 500.0])
            .with_title("Monitor Sites"),
        ..Default::default()
    };

    eframe::run_native(
        "Monitor Sites",
        opcoes,
        Box::new(|_cc| Box::new(AppGui::default())),
    ).unwrap_or_else(|e| {
        crate::log::erro(&format!("erro ao iniciar GUI: {}", e));
        panic!();
    })
}

impl eframe::App for AppGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.inicializado {
            self.carregar_hosts();
            self.inicializado = true;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Monitor Sites");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("X").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            })
            ui.separator();

            ui.label("Hosts monitorados:");
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for host in &self.hosts {
                        ui.label(host);
                    }
                    if self.hosts.is_empty() {
                        ui.label("nenhum host cadastrado");
                    }
                });

            ui.separator();

            // botão de adicionar
            if ui.button("+ Adicionar host").clicked() {
                self.mostrar_formulario = true;
            }
        });

        //janela de formulário
        if self.mostrar_formulario {
            egui::Window::new("Adicionar host")
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Domínio: ");
                    ui.text_edit_singleline(&mut self.dominio);

                    ui.label("IP:");
                    ui.text_edit_singleline(&mut self.ip);

                    ui.label("Portas (separadas por vírgula):");

                    ui.text_edit_singleline(&mut self.portas);

                    ui.separator();

                    // checkboxes portas padrão
                    ui.label("Portas padrão:");
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.porta_22, "22");
                        ui.checkbox(&mut self.porta_22, "80");
                        ui.checkbox(&mut self.porta_22, "443");
                    });

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.porta_22, "3306");
                        ui.checkbox(&mut self.porta_22, "5432");
                        ui.checkbox(&mut self.porta_22, "8080");
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Cancelar").clicked() {
                            self.mostrar_formulario = false;
                            self.dominio.clear();
                            self.ip.clear();
                            self.portas.clear();
                        }
                        if ui.button("Adicionar").clicked() {
                            let json = format!(
                                r#"{{"comando":"adicionar_host", "dominio":"{}", "ip":"{}"}}"#,
                                self.dominio, self.ip
                            );

                            println!("enviando: {}", json);

                            match crate::gui::socket::enviar_comando( &json) {
                                Some(r) => println!("resposta: {}", r),
                                None => println!("enviar_comando retornou None")
                            }

                            self.mostrar_formulario = false;
                            self.dominio.clear();
                            self.ip.clear();
                            self.portas.clear();
                        }
                    })
                });
        }
    }
}