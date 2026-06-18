use eframe::egui;
use gtk;
use tray_icon::Icon;
use tray_icon::{
    menu::{Menu, MenuItem},
    TrayIcon, TrayIconBuilder,
};

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
    host_selecionado: Option<(i64, String)>
}

impl Default for AppGui {
    fn default() -> Self {
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
            host_selecionado: None,
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
    let icone = Icon::from_rgba(imagem.into_raw(), largura, altura).expect("erro ao criar ícone");

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
    )
    .unwrap_or_else(|e| {
        crate::log::erro(&format!("erro ao iniciar GUI: {}", e));
        panic!();
    })
}

impl AppGui {
    fn carregar_hosts(&mut self) {
        let json = r#"{"comando": "listar_hosts"}"#;
        if let Some(resposta) = crate::gui::socket::enviar_comando(json) {
            if let Ok(valor) = serde_json::from_str::<serde_json::Value>(&resposta) {
                if valor["ok"].as_bool().unwrap_or(false) {
                    if let Some(dados) = valor["dados"].as_str() {
                        if dados.is_empty() {
                            self.hosts = Vec::new();
                        } else {
                            self.hosts = dados.split('|').map(|s| s.to_string()).collect();
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for AppGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.inicializado {
            self.carregar_hosts();
            self.inicializado = true;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Monitor Sites");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("X").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    };
                })
            });
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
                        ui.checkbox(&mut self.porta_80, "80");
                        ui.checkbox(&mut self.porta_443, "443");
                    });

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.porta_3306, "3306");
                        ui.checkbox(&mut self.porta_5432, "5432");
                        ui.checkbox(&mut self.porta_8080, "8080");
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
                            r#"{{"comando": "adicionar_host", "dominio":"{}", "ip":"{}"}}"#,
                            self.dominio, self.ip
                        );
                        println!("enviando: {}", json);
                        match crate::gui::socket::enviar_comando(&json) {
                            Some(resposta) => {
                                // extrai o id do host criado da resposta
                                if let Ok(valor) = serde_json::from_str::<serde_json::Value>(&resposta) {
                                    if let Some(id_str) = valor["dados"].as_str() {
                                        if let Ok(host_id) = id_str.parse::<i64>(){
                                                // envia porta para cada checkbox marcada
                                            let portas_selecionadas = [
                                                (self.porta_22, 22u16),
                                                (self.porta_80, 80),
                                                (self.porta_443, 443),
                                                (self.porta_3306, 3306),
                                                (self.porta_5432, 5432),
                                                (self.porta_8080, 8080)
                                            ];
                                            for (marcada, numero) in portas_selecionadas {
                                                if marcada {
                                                    let json_porta = format!(
                                                        r#"{{"comando":"adicionar_porta", "host_id":{}, "porta":{}}}"#,
                                                        host_id, numero
                                                    );
                                                    crate::gui::socket::enviar_comando(&json_porta);
                                                }
                                            }
                                            self.carregar_hosts();
                                        }
                                        
                                    }
                                }
                            }
                            None => println!("falhou ao enviar")
                        }
                        self.mostrar_formulario = false;
                        self.dominio.clear();
                        self.ip.clear();
                        self.portas.clear();
                        self.porta_22 = false;
                        self.porta_80 = false;
                        self.porta_443 = false;
                        self.porta_3306 = false;
                        self.porta_5432 = false;
                        self.porta_8080 = false;
                        }
                    })
                });
        }
    }
}
