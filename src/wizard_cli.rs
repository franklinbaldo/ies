//! Assistente interativo (wizard) nativo em Rust para configuração do IES.
//! Suporta instalações públicas e personalizadas, com senha protegida no Keyring (serviço "ies").

use clap::Args;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Args)]
pub(crate) struct WizardArgs {
    #[arg(long, help = "Nome do perfil a configurar (pula seleção inicial)")]
    perfil: Option<String>,
}

struct InstalacaoPreset {
    slug: &'static str,
    nome: &'static str,
    origem: &'static str,
    usuario: &'static str,
    senha: &'static str,
    orgao: &'static str,
}

const PRESETS: &[InstalacaoPreset] = &[
    InstalacaoPreset {
        slug: "unb",
        nome: "UnB 4.0.9 (Treinamento)",
        origem: "https://sei.treinamento.unb.br/sei",
        usuario: "aluno1",
        senha: "aluno1",
        orgao: "UnB",
    },
    InstalacaoPreset {
        slug: "pen1",
        nome: "PEN1 5.0.4 / Tramita (Homologação)",
        origem: "https://sei.orgao1.tramita.processoeletronico.gov.br/sei",
        usuario: "orgao1",
        senha: "orgao1",
        orgao: "ORGAO1",
    },
    InstalacaoPreset {
        slug: "go",
        nome: "Goiás 5.0.3 (Treinamento)",
        origem: "https://seitreinamento.go.gov.br/sei",
        usuario: "aluno05",
        senha: "abc123456",
        orgao: "SEAD",
    },
    InstalacaoPreset {
        slug: "ufrj",
        nome: "UFRJ (Treinamento)",
        origem: "https://teste.sei.tic.ufrj.br/sei",
        usuario: "servidor1",
        senha: "servidor1",
        orgao: "UFRJ",
    },
];

fn ler_linha(prompt: &str) -> String {
    print!("{prompt}");
    let _ = io::stdout().flush();
    let mut buffer = String::new();
    let _ = io::stdin().lock().read_line(&mut buffer);
    buffer.trim().to_string()
}

fn normalizar_origem(url: &str) -> String {
    let mut u = url.trim().trim_end_matches('/').to_string();
    if !u.starts_with("http://") && !u.starts_with("https://") {
        u = format!("https://{u}");
    }
    // Remove controladores conhecidos
    for entry in &["/sip/login.php", "/sei/controlador.php", "/sei/inicializar.php"] {
        if let Some(pos) = u.to_lowercase().find(entry) {
            u.truncate(pos);
            break;
        }
    }
    if !u.ends_with("/sei") {
        format!("{u}/sei")
    } else {
        u
    }
}

fn extrair_orgaos_do_html(html: &str) -> Vec<(String, String)> {
    let html_lower = html.to_lowercase();
    let mut pos_select = 0;
    let mut orgaos = Vec::new();

    while let Some(idx) = html_lower[pos_select..].find("<select") {
        let abs_start = pos_select + idx;
        let fim_tag = match html_lower[abs_start..].find('>') {
            Some(f) => abs_start + f,
            None => break,
        };
        let tag_select = &html_lower[abs_start..=fim_tag];
        if tag_select.contains("selorgao") {
            let fim_select = match html_lower[fim_tag..].find("</select>") {
                Some(f) => fim_tag + f,
                None => html.len(),
            };
            let corpo_select = &html[fim_tag + 1..fim_select];
            let mut pos_opt = 0;
            let corpo_lower = corpo_select.to_lowercase();

            while let Some(opt_idx) = corpo_lower[pos_opt..].find("<option") {
                let abs_opt = pos_opt + opt_idx;
                let fim_opt_tag = match corpo_select[abs_opt..].find('>') {
                    Some(f) => abs_opt + f,
                    None => break,
                };
                let opt_header = &corpo_select[abs_opt..fim_opt_tag];
                let mut val = String::new();
                if let Some(v_idx) = opt_header.to_lowercase().find("value=") {
                    let resto = &opt_header[v_idx + 6..];
                    let delimitador = resto.chars().next().unwrap_or(' ');
                    if delimitador == '"' || delimitador == '\'' {
                        val = resto[1..].split(delimitador).next().unwrap_or("").to_string();
                    } else {
                        val = resto.split_whitespace().next().unwrap_or("").to_string();
                    }
                }

                let fim_conteudo = match corpo_lower[fim_opt_tag + 1..].find("</option") {
                    Some(f) => fim_opt_tag + 1 + f,
                    None => match corpo_lower[fim_opt_tag + 1..].find("<option") {
                        Some(f) => fim_opt_tag + 1 + f,
                        None => corpo_select.len(),
                    },
                };
                let texto = corpo_select[fim_opt_tag + 1..fim_conteudo]
                    .trim()
                    .replace("&nbsp;", " ")
                    .replace("&amp;", "&");
                let texto = texto.split_whitespace().collect::<Vec<_>>().join(" ");

                if !val.is_empty() && val != "null" && !texto.is_empty() {
                    orgaos.push((val, texto));
                }
                pos_opt = fim_conteudo;
            }
            break;
        }
        pos_select = fim_tag + 1;
    }
    orgaos
}

fn sondar_instalacao(origem: &str) -> (Option<String>, Vec<(String, String)>) {
    let mut url_atual = origem.to_string();
    let mut host = "";
    if let Some(pos) = origem.find("://") {
        let sem_esq = &origem[pos + 3..];
        host = sem_esq.split('/').next().unwrap_or(sem_esq);
    }

    let mut sigla_sip = None;
    let mut html_final = String::new();

    print!("  Sondando login e catálogo de órgãos... ");
    let _ = io::stdout().flush();

    for _ in 0..6 {
        let resp = match crate::transporte::cliente_https::requisitar("GET", &url_atual, "", None) {
            Ok(r) => r,
            Err(_) => break,
        };

        if (300..400).contains(&resp.status) {
            if let Some(loc) = resp.cabecalho("location") {
                let proxima = if loc.starts_with("http://") || loc.starts_with("https://") {
                    loc.to_string()
                } else if loc.starts_with('/') {
                    format!("https://{host}{loc}")
                } else {
                    format!("{url_atual}/{loc}")
                };
                if proxima.contains("sigla_orgao_sistema=") {
                    if let Some(pos) = proxima.find("sigla_orgao_sistema=") {
                        let pedaco = &proxima[pos + 20..];
                        let sigla = pedaco.split('&').next().unwrap_or(pedaco);
                        sigla_sip = Some(sigla.to_string());
                    }
                }
                url_atual = proxima;
                continue;
            }
        }

        if resp.status == 200 {
            use std::io::Read;
            let mut buf = Vec::new();
            let mut corpo = resp.corpo;
            let _ = corpo.read_to_end(&mut buf);
            html_final = String::from_utf8_lossy(&buf).into_owned();
            break;
        }
        break;
    }

    println!("\x1b[1;32mconcluído!\x1b[0m");
    let orgaos = extrair_orgaos_do_html(&html_final);
    (sigla_sip, orgaos)
}

fn escolher_orgao(orgaos: &[(String, String)]) -> String {
    if orgaos.is_empty() {
        return ler_linha("Sigla do Órgão (ex: PGE): ");
    }
    if orgaos.len() == 1 {
        let (_, rotulo) = &orgaos[0];
        println!("  Órgão único detectado: \x1b[1;32m{rotulo}\x1b[0m");
        return rotulo.clone();
    }

    println!("\n\x1b[1mÓrgãos disponíveis nesta instalação ({}):\x1b[0m", orgaos.len());
    let largura_num = orgaos.len().to_string().len();
    for (i, (id_val, rotulo)) in orgaos.iter().enumerate() {
        println!("  {:>largura$}) {} (id {})", i + 1, rotulo, id_val, largura = largura_num);
    }
    println!();

    loop {
        let entrada = ler_linha(&format!("Escolha o órgão (1-{}) ou digite a sigla: ", orgaos.len()));
        let entrada = entrada.trim();
        if entrada.is_empty() {
            continue;
        }
        if let Ok(num) = entrada.parse::<usize>() {
            if num >= 1 && num <= orgaos.len() {
                return orgaos[num - 1].1.clone();
            }
        }
        let correspondencias: Vec<&(String, String)> = orgaos
            .iter()
            .filter(|(_, r)| r.eq_ignore_ascii_case(entrada))
            .collect();
        if correspondencias.len() == 1 {
            return correspondencias[0].1.clone();
        }
        let parciais: Vec<&(String, String)> = orgaos
            .iter()
            .filter(|(_, r)| r.to_lowercase().contains(&entrada.to_lowercase()))
            .collect();
        if parciais.len() == 1 {
            return parciais[0].1.clone();
        } else if parciais.len() > 1 {
            let nomes: Vec<&str> = parciais.iter().take(5).map(|(_, r)| r.as_str()).collect();
            println!("Mais de um órgão encontrado: {}...", nomes.join(", "));
        } else {
            println!("Nenhum órgão corresponde a '{entrada}'.");
        }
    }
}

pub(crate) fn executar_wizard(args: &WizardArgs, _json_mode: bool) -> ExitCode {
    println!("\x1b[1;36m==================================================\x1b[0m");
    println!("\x1b[1;36m       IES — Assistente de Configuração (Rust)    \x1b[0m");
    println!("\x1b[1;36m       (Cofre nativo: Keyring serviço 'ies')      \x1b[0m");
    println!("\x1b[1;36m==================================================\x1b[0m\n");

    println!("Escolha a instalação do SEI:");
    for (i, p) in PRESETS.iter().enumerate() {
        println!("  \x1b[1;33m{}\x1b[0m) {}", i + 1, p.nome);
    }
    println!("  \x1b[1;33m5\x1b[0m) Conectar ao meu SEI (personalizado / corporativo)\n");

    let escolha = ler_linha("Opção [1-5] (padrão: 5): ");
    let escolha = if escolha.is_empty() { "5" } else { &escolha };

    let (slug, origem, usuario, senha, orgao) = match escolha {
        "1" | "2" | "3" | "4" => {
            let idx = escolha.parse::<usize>().unwrap() - 1;
            let p = &PRESETS[idx];
            println!("\nUsando predefinição: \x1b[1m{}\x1b[0m", p.nome);
            (
                p.slug.to_string(),
                p.origem.to_string(),
                p.usuario.to_string(),
                p.senha.to_string(),
                p.orgao.to_string(),
            )
        }
        "5" => {
            let raw_url = ler_linha("\nCole a URL do seu SEI (ex: https://sei.sistemas.ro.gov.br): ");
            let orig = normalizar_origem(&raw_url);
            println!("  Origem configurada: \x1b[1;32m{orig}\x1b[0m");

            let (sigla_sip, orgaos) = sondar_instalacao(&orig);
            if let Some(sip) = &sigla_sip {
                println!("  Ambiente SIP:       \x1b[1;36m{sip}\x1b[0m");
            }

            let o = escolher_orgao(&orgaos);
            let u = ler_linha("Usuário (login): ");

            print!("Senha: ");
            let _ = io::stdout().flush();
            let s = match rpassword::read_password() {
                Ok(pwd) if !pwd.trim().is_empty() => pwd,
                _ => ler_linha(""),
            };

            let prefixo = sigla_sip.as_deref().unwrap_or("sei").to_lowercase();
            let perfil_sug = format!("{prefixo}-{}", o.to_lowercase());
            let final_slug = if let Some(p) = &args.perfil {
                p.clone()
            } else {
                let p_nome = ler_linha(&format!("Nome do perfil local [{perfil_sug}]: "));
                if p_nome.is_empty() { perfil_sug } else { p_nome }
            };
            (final_slug, orig, u, s, o)
        }
        _ => {
            eprintln!("Opção inválida.");
            return ExitCode::from(1);
        }
    };

    // Pergunta sobre proteção no Keyring
    let salvar_cofre = ler_linha("\nDeseja salvar a senha com segurança no cofre do sistema (Keyring 'ies')? (S/N) [S]: ");
    let no_cofre = salvar_cofre.is_empty() || salvar_cofre.to_lowercase().starts_with('s');

    if no_cofre {
        match crate::cofre::salvar_senha(&usuario, &origem, &senha) {
            Ok(()) => println!("\x1b[1;32m✓ Senha gravada com segurança no cofre nativo (serviço 'ies').\x1b[0m"),
            Err(e) => eprintln!("Aviso: Falha ao gravar no cofre ({e}). Seguindo com arquivo privado."),
        }
    }

    // Diretório base
    let home = match std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        Some(h) => PathBuf::from(h),
        None => PathBuf::from("."),
    };
    let dir = home.join(".ies").join(&slug);
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("Falha ao criar diretório {}: {e}", dir.display());
        return ExitCode::from(1);
    }

    let arq_origem = dir.join("origem");
    let arq_cookie = dir.join("cookie");
    let arq_url_controle = dir.join("url-controle");
    let arq_credenciais = dir.join("credenciais");
    let arq_config = dir.join("ies-config.json");

    let _ = fs::write(&arq_origem, &origem);
    if !arq_cookie.exists() {
        let _ = fs::write(&arq_cookie, "");
    }
    if !arq_url_controle.exists() {
        let _ = fs::write(&arq_url_controle, "");
    }

    // Grava credenciais (se a senha foi pro cofre, podemos gravar senha vazia no arquivo ou a senha completa)
    let cred_content = format!("usuario={usuario}\nsenha={senha}\norgao={orgao}\n");
    let _ = fs::write(&arq_credenciais, &cred_content);

    // Remove config anterior se houver
    let _ = fs::remove_file(&arq_config);

    // Executa ies setup
    let arquivos = crate::configuracao::ArquivosDeAcesso {
        url_controle: arq_url_controle.clone(),
        cookie: arq_cookie.clone(),
        origem: arq_origem.clone(),
    };
    if let Err(e) = crate::configuracao::salvar_arquivos_de_acesso(&arq_config, &arquivos) {
        eprintln!("Falha no ies setup: {e}");
        return ExitCode::from(1);
    }
    println!("\x1b[1;32m✓ Configuração gravada em: {}\x1b[0m", arq_config.display());

    // Scripts de conveniência
    let script_ps1 = dir.join("ativar.ps1");
    let _ = fs::write(
        &script_ps1,
        format!(
            "$env:IES_CONFIG_FILE = \"{}\"\nif (Test-Path \"{}\") {{ $env:IES_CREDENCIAIS_FILE = \"{}\" }}\nWrite-Host \"IES ativo: {}\" -ForegroundColor Green\n",
            arq_config.display(),
            arq_credenciais.display(),
            arq_credenciais.display(),
            slug
        ),
    );

    let testar = ler_linha("\nDeseja testar a conexão com 'ies login' agora? (S/N) [S]: ");
    if testar.is_empty() || testar.to_lowercase().starts_with('s') {
        println!("\nConectando e autenticando no SEI...");
        // Define temporariamente para o login
        unsafe {
            std::env::set_var("IES_CONFIG_FILE", &arq_config);
            std::env::set_var("IES_CREDENCIAIS_FILE", &arq_credenciais);
        }
        let res = crate::sessao_login_cli::executar_login(false, false, None);
        if res == ExitCode::SUCCESS && no_cofre {
            let limpar = ler_linha("\nDeseja apagar a senha em texto claro do disco agora? (S/N) [S]: ");
            if limpar.is_empty() || limpar.to_lowercase().starts_with('s') {
                let _ = fs::remove_file(&arq_credenciais);
                println!("\x1b[1;32m✓ Arquivo 'credenciais' removido do disco. Sessão continua no cookie!\x1b[0m");
                println!("  (A senha permanece protegida no Keyring para logins futuros).");
            }
        }
    }

    println!("\n--------------------------------------------------");
    println!("Para ativar este perfil no seu terminal:");
    println!("  PowerShell:");
    println!("    . {}", script_ps1.display());
    println!("    ou:");
    println!("    $env:IES_CONFIG_FILE = \"{}\"", arq_config.display());
    println!("\nComandos de exemplo:");
    println!("  ies status");
    println!("  ies unidade atual");
    println!("  ies unidade recebidos");
    println!("  ies processo <NUP>");
    println!("--------------------------------------------------\n");

    ExitCode::SUCCESS
}
