//! Assistente interativo (wizard) nativo em Rust para configuração do IES.
//! Grava senhas exclusivamente no cofre seguro do SO (Keyring escopado: "ies:<instancia>").
//! O arquivo privado no disco nunca armazena a senha em texto claro.

use clap::Args;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use crate::wizard_sondagem::{normalizar_origem, sondar_instalacao};

#[derive(Debug, Args)]
pub(crate) struct WizardArgs {
    #[arg(long, help = "Nome do perfil a configurar")]
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

fn ler_senha_segura() -> Result<String, String> {
    print!("Senha: ");
    let _ = io::stdout().flush();
    match rpassword::read_password() {
        Ok(pwd) => {
            let trim = pwd.trim().to_string();
            if trim.is_empty() {
                Err("A senha não pode ser vazia.".to_string())
            } else {
                Ok(trim)
            }
        }
        Err(e) => Err(format!("Falha ao ler senha sem eco: {e}")),
    }
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

    println!(
        "\n\x1b[1mÓrgãos disponíveis nesta instalação ({}):\x1b[0m",
        orgaos.len()
    );
    let largura_num = orgaos.len().to_string().len();
    for (i, (id_val, rotulo)) in orgaos.iter().enumerate() {
        println!(
            "  {:>largura$}) {} (id {})",
            i + 1,
            rotulo,
            id_val,
            largura = largura_num
        );
    }
    println!();

    loop {
        let entrada = ler_linha(&format!(
            "Escolha o órgão (1-{}) ou digite a sigla: ",
            orgaos.len()
        ));
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
    println!("\x1b[1;36m       IES — Assistente de Configuração           \x1b[0m");
    println!("\x1b[1;36m       (Cofre Nativo do SO: Keyring)              \x1b[0m");
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
            let idx = match escolha.parse::<usize>() {
                Ok(n) if n >= 1 && n <= PRESETS.len() => n - 1,
                _ => return ExitCode::from(1),
            };
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
            let raw_url =
                ler_linha("\nCole a URL do seu SEI (ex: https://sei.sistemas.ro.gov.br): ");
            let orig = match normalizar_origem(&raw_url) {
                Ok(u) => u,
                Err(e) => {
                    eprintln!("Erro: {e}");
                    return ExitCode::from(1);
                }
            };
            println!("  Origem configurada: \x1b[1;32m{orig}\x1b[0m");

            let consentimento = ler_linha(
                "\nDeseja sondar a instalação na rede para catálogo de órgãos? (S/N) [S]: ",
            );
            let (sigla_sip, orgaos) =
                if consentimento.is_empty() || consentimento.to_lowercase().starts_with('s') {
                    print!("  Sondando login e catálogo de órgãos... ");
                    let _ = io::stdout().flush();
                    match sondar_instalacao(&orig) {
                        Ok((sip, orgs)) => {
                            println!("\x1b[1;32mconcluído!\x1b[0m");
                            (sip, orgs)
                        }
                        Err(motivo) => {
                            println!("\x1b[1;33mignorado ({motivo})\x1b[0m");
                            (None, Vec::new())
                        }
                    }
                } else {
                    println!("  (Configuração manual/offline selecionada)");
                    (None, Vec::new())
                };

            if let Some(sip) = &sigla_sip {
                println!("  Ambiente SIP:       \x1b[1;36m{sip}\x1b[0m");
            }

            let o = escolher_orgao(&orgaos);
            let u = ler_linha("Usuário (login): ");
            if u.is_empty() {
                eprintln!("Erro: Usuário não pode ser vazio.");
                return ExitCode::from(1);
            }

            let s = match ler_senha_segura() {
                Ok(pwd) => pwd,
                Err(e) => {
                    eprintln!("Erro: {e}");
                    return ExitCode::from(1);
                }
            };

            let prefixo = sigla_sip.as_deref().unwrap_or("sei").to_lowercase();
            let perfil_sug = format!("{prefixo}-{}", o.to_lowercase());
            let final_slug = if let Some(p) = &args.perfil {
                p.clone()
            } else {
                let p_nome = ler_linha(&format!("Nome do perfil local [{perfil_sug}]: "));
                if p_nome.is_empty() {
                    perfil_sug
                } else {
                    p_nome
                }
            };
            (final_slug, orig, u, s, o)
        }
        _ => {
            eprintln!("Opção inválida.");
            return ExitCode::from(1);
        }
    };

    // Grava a senha no cofre nativo canônico (NUNCA em disco)
    let servico_canonica = crate::cofre::servico_do_cofre(&origem);
    if let Err(e) = crate::cofre::salvar_senha(&usuario, &origem, &senha) {
        eprintln!("Falha crítica ao gravar credencial no cofre nativo: {e}");
        return ExitCode::from(1);
    }
    println!(
        "\x1b[1;32m✓ Senha gravada com segurança no cofre nativo ('{servico_canonica}').\x1b[0m"
    );

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

    if let Err(e) = fs::write(&arq_origem, &origem) {
        eprintln!("Falha ao gravar arquivo origem: {e}");
        return ExitCode::from(1);
    }
    if !arq_cookie.exists() && fs::write(&arq_cookie, "").is_err() {
        eprintln!("Falha ao inicializar arquivo cookie.");
        return ExitCode::from(1);
    }
    if !arq_url_controle.exists() && fs::write(&arq_url_controle, "").is_err() {
        eprintln!("Falha ao inicializar arquivo url-controle.");
        return ExitCode::from(1);
    }

    // O arquivo 'credenciais' contém APENAS usuário e órgão. A senha NUNCA vai ao disco.
    let cred_content = format!("usuario={usuario}\norgao={orgao}\n");
    if let Err(e) = fs::write(&arq_credenciais, &cred_content) {
        eprintln!("Falha ao gravar arquivo de credenciais: {e}");
        return ExitCode::from(1);
    }

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
    println!(
        "\x1b[1;32m✓ Configuração gravada em: {}\x1b[0m",
        arq_config.display()
    );

    // Scripts de ativação exportando referências
    let script_ps1 = dir.join("ativar.ps1");
    let ps1_conteudo = format!(
        "$env:IES_CONFIG_FILE = \"{}\"\n$env:IES_CREDENCIAIS_FILE = \"{}\"\n$env:IES_USUARIO = \"{}\"\n$env:IES_ORGAO = \"{}\"\nWrite-Host \"IES ativo: {}\" -ForegroundColor Green\n",
        arq_config.display(),
        arq_credenciais.display(),
        usuario,
        orgao,
        slug
    );
    if let Err(e) = fs::write(&script_ps1, ps1_conteudo) {
        eprintln!("Falha ao gravar script de ativação PowerShell: {e}");
        return ExitCode::from(1);
    }

    let script_sh = dir.join("ativar.sh");
    let sh_conteudo = format!(
        "export IES_CONFIG_FILE=\"{}\"\nexport IES_CREDENCIAIS_FILE=\"{}\"\nexport IES_USUARIO=\"{}\"\nexport IES_ORGAO=\"{}\"\necho \"IES ativo: {}\"\n",
        arq_config.display(),
        arq_credenciais.display(),
        usuario,
        orgao,
        slug
    );
    let _ = fs::write(&script_sh, sh_conteudo);

    let testar = ler_linha("\nDeseja testar a conexão com 'ies login' agora? (S/N) [S]: ");
    if testar.is_empty() || testar.to_lowercase().starts_with('s') {
        println!("\nConectando e autenticando no SEI...");
        unsafe {
            std::env::set_var("IES_CONFIG_FILE", &arq_config);
            std::env::set_var("IES_CREDENCIAIS_FILE", &arq_credenciais);
            std::env::set_var("IES_USUARIO", &usuario);
            std::env::set_var("IES_ORGAO", &orgao);
        }
        let _ = crate::sessao_login_cli::executar_login(false, false, None);
    }

    println!("\n--------------------------------------------------");
    println!(
        "Para ativar este perfil no terminal:\n  PowerShell:\n    . {}\n    ou:\n    $env:IES_CONFIG_FILE = \"{}\"\n    $env:IES_CREDENCIAIS_FILE = \"{}\"",
        script_ps1.display(),
        arq_config.display(),
        arq_credenciais.display()
    );
    println!(
        "\nComandos de exemplo:\n  ies status\n  ies unidade atual\n  ies unidade recebidos\n  ies processo <NUP>"
    );
    println!("--------------------------------------------------\n");
    ExitCode::SUCCESS
}
