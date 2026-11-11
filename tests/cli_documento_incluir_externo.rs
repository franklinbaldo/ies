//! #332: comando externo inspeciona PDF offline; --confirmar exige sessão.
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static PROXIMO: AtomicUsize = AtomicUsize::new(0);

fn executar(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env_remove("IES_CONFIG_FILE")
        .env("IES_URL_CONTROLE_FILE", "/nao/existe/url")
        .env("IES_COOKIE_FILE", "/nao/existe/cookie")
        .env("IES_ORIGEM_FILE", "/nao/existe/origem")
        .output()
        .expect("ies deve iniciar")
}

#[path = "support/falha.rs"]
mod falha_cli;
use falha_cli::falha;

fn json(saida: &Output) -> serde_json::Value {
    serde_json::from_slice(&saida.stdout).expect("saída deve ser JSON")
}

struct PdfTeste(PathBuf);

impl PdfTeste {
    fn novo() -> Self {
        Self::com_conteudo("pdf", b"%PDF-1.4\n1 0 obj\n%%EOF\n")
    }
    fn com_conteudo(extensao: &str, conteudo: &[u8]) -> Self {
        let nome = format!(
            "ies-332-cli-{}-{}.{}",
            std::process::id(),
            PROXIMO.fetch_add(1, Ordering::Relaxed),
            extensao,
        );
        let caminho = std::env::temp_dir().join(nome);
        fs::write(&caminho, conteudo).unwrap();
        Self(caminho)
    }
    fn caminho(&self) -> &str {
        self.0.to_str().unwrap()
    }
}

impl Drop for PdfTeste {
    fn drop(&mut self) {
        fs::remove_file(&self.0).unwrap();
    }
}

#[test]
fn bdd_332_01_pdf_local_publica_inspecao_sem_sessao_ou_http() {
    let pdf = PdfTeste::novo();
    let saida = executar(&[
        "--json",
        "documento",
        "incluir-externo",
        "--processo",
        "23106.000001/2026-11",
        "--arquivo",
        pdf.caminho(),
        "--tipo",
        "Anexo",
    ]);
    assert_eq!(saida.status.code(), Some(0), "{:?}", saida.stderr);
    let valor = json(&saida);
    assert_eq!(valor["escopo"], "inspecao_local");
    assert_eq!(valor["processo"], "23106.000001/2026-11");
    assert_eq!(valor["tipo_solicitado"], "Anexo");
    assert_eq!(
        valor["arquivo"]["nome"],
        pdf.0.file_name().unwrap().to_str().unwrap()
    );
    assert_eq!(valor["arquivo"]["tamanho_bytes"], 23);
    assert_eq!(
        valor["arquivo"]["sha256"],
        "49286c45c90723857946342e0610a2b28974f176218f77f79efacc34cc4567bc"
    );
    assert_eq!(valor["cadastro_sei_observado"], false);
    assert_eq!(valor["previa_remota"], false);
    assert_eq!(valor["confirmado"], false);
    assert_eq!(valor["requisicoes"], 0);
    assert_eq!(valor["escrita_remota"], false);
}

#[test]
fn bdd_332_05_pdf_falso_e_extensao_incorreta_recusados_antes_da_rede() {
    let falso = PdfTeste::com_conteudo("pdf", b"texto, nao PDF");
    let extensao = PdfTeste::com_conteudo("txt", b"%PDF-1.4\n1 0 obj\n%%EOF\n");
    // A inspeção local é pré-condição até mesmo da consulta e da confirmação.
    for (arquivo, causa) in [(&falso, "Conteudo"), (&extensao, "Extensao")] {
        for modo in [None, Some("--consultar-sei"), Some("--confirmar")] {
            let mut args = vec![
                "--json",
                "documento",
                "incluir-externo",
                "--processo",
                "23106.000001/2026-11",
                "--arquivo",
                arquivo.caminho(),
                "--tipo",
                "Anexo",
            ];
            if let Some(modo) = modo {
                args.push(modo);
            }
            // Sem arquivos de sessão válidos, só a inspeção local poderia
            // produzir este erro: nenhuma requisição foi feita.
            let saida = executar(&args);
            falha(&saida, causa);
        }
    }
}

#[cfg(unix)]
#[test]
fn bdd_332_05_caminho_simbolico_ou_diretorio_sao_recusados() {
    use std::os::unix::fs::symlink;
    let pdf = PdfTeste::novo();
    let link = pdf.0.with_extension("link.pdf");
    let diretorio = pdf.0.with_extension("diretorio.pdf");
    symlink(&pdf.0, &link).unwrap();
    fs::create_dir(&diretorio).unwrap();
    for (caminho, causa) in [(&link, "LinkSimbolico"), (&diretorio, "NaoRegular")] {
        let saida = executar(&[
            "--json",
            "documento",
            "incluir-externo",
            "--processo",
            "23106.000001/2026-11",
            "--arquivo",
            caminho.to_str().unwrap(),
            "--tipo",
            "Anexo",
        ]);
        falha(&saida, causa);
    }
    fs::remove_file(link).unwrap();
    fs::remove_dir(diretorio).unwrap();
}

#[test]
fn confirmar_vai_a_sessao_e_sem_ela_falha_antes_da_rede() {
    let pdf = PdfTeste::novo();
    let saida = executar(&[
        "--json",
        "documento",
        "incluir-externo",
        "--processo",
        "23106.000001/2026-11",
        "--arquivo",
        pdf.caminho(),
        "--tipo",
        "Anexo",
        "--confirmar",
    ]);
    // A escrita não depende de chave de ensaio: --confirmar segue para a
    // sessão; sem arquivos de sessão, falha antes de qualquer requisição.
    assert_eq!(saida.status.code(), Some(1));
}

#[test]
fn arquivo_local_ausente_e_recusado_sem_sessao() {
    let saida = executar(&[
        "--json",
        "documento",
        "incluir-externo",
        "--processo",
        "23106.000001/2026-11",
        "--arquivo",
        "/arquivo/inexistente/nao.pdf",
        "--tipo",
        "Anexo",
    ]);
    falha(&saida, "Ausente");
}

#[test]
fn nome_pdf_incompativel_rejeitado_com_erro_tipado_antes_da_rede() {
    let nome = format!(
        "ies-332-{}-{}&#1.pdf",
        std::process::id(),
        PROXIMO.fetch_add(1, Ordering::Relaxed)
    );
    let pdf = PdfTeste(std::env::temp_dir().join(nome));
    fs::write(&pdf.0, b"%PDF-1.4\n1 0 obj\n%%EOF\n").unwrap();
    let saida = executar(&[
        "--json",
        "documento",
        "incluir-externo",
        "--processo",
        "23106.000001/2026-11",
        "--arquivo",
        pdf.caminho(),
        "--tipo",
        "Anexo",
    ]);
    falha(&saida, "NomeInvalido");
}

#[test]
fn tipo_com_controle_direcional_nao_pode_enganar_a_saida_humana() {
    // O marcador RTL é formatador Unicode, não char::is_control.
    // Deve ser barrado antes da sessão/rede mesmo se o PDF for válido.
    let pdf = PdfTeste::novo();
    let saida = executar(&[
        "--json",
        "documento",
        "incluir-externo",
        "--processo",
        "23106.000001/2026-11",
        "--arquivo",
        pdf.caminho(),
        "--tipo",
        "An\u{202e}exo",
    ]);
    assert_eq!(saida.status.code(), Some(2));
}

#[test]
fn tipo_vazio_nao_acessa_arquivo_nem_sessao() {
    let saida = executar(&[
        "--json",
        "documento",
        "incluir-externo",
        "--processo",
        "23106.000001/2026-11",
        "--arquivo",
        "/arquivo/inexistente/nao.pdf",
        "--tipo",
        " ",
    ]);
    assert_eq!(saida.status.code(), Some(2));
}

#[test]
fn numero_invalido_expoe_o_tipo_da_causa() {
    let pdf = PdfTeste::novo();
    let saida = executar(&[
        "--json",
        "documento",
        "incluir-externo",
        "--processo",
        " ",
        "--arquivo",
        pdf.caminho(),
        "--tipo",
        "Anexo",
    ]);
    assert_eq!(saida.status.code(), Some(2));
    assert!(
        saida.stdout.is_empty(),
        "uso inválido é recusado pelo clap, sem JSON"
    );
}

#[test]
fn dado_pedido_de_previa_web_quando_sem_sessao_entao_nao_finge_inspecao_local() {
    // BDD #332: prévia Web deve consultar o SEI (aqui sessão propositalmente ausente).
    let pdf = PdfTeste::novo();
    let saida = executar(&[
        "--json",
        "documento",
        "incluir-externo",
        "--processo",
        "23106.000001/2026-11",
        "--arquivo",
        pdf.caminho(),
        "--tipo",
        "Anexo",
        "--consultar-sei",
    ]);
    assert_eq!(saida.status.code(), Some(1), "{:?}", saida.stderr);
    // Sem inspeção local fingida: nada no stdout, só o erro tipado.
    assert_eq!(saida.stdout, b"");
    let erro = String::from_utf8_lossy(&saida.stderr);
    assert!(
        erro.starts_with("Error: ArquivoPrivadoAusenteOuInacessivel"),
        "{erro}"
    );
}

#[test]
fn dado_confirmar_com_previa_web_quando_sem_sessao_entao_falha_antes_da_rede() {
    let pdf = PdfTeste::novo();
    let saida = executar(&[
        "--json",
        "documento",
        "incluir-externo",
        "--processo",
        "23106.000001/2026-11",
        "--arquivo",
        pdf.caminho(),
        "--tipo",
        "Anexo",
        "--consultar-sei",
        "--confirmar",
    ]);
    // A escrita não depende de chave de ensaio: --confirmar segue para a
    // sessão; sem arquivos de sessão, falha antes de qualquer requisição.
    assert_eq!(saida.status.code(), Some(1));
}

#[test]
fn inspecao_local_humana_mantem_aviso_sem_sessao_ou_rede() {
    let pdf = PdfTeste::novo();
    let saida = executar(&[
        "documento",
        "incluir-externo",
        "--processo",
        "23106.000001/2026-11",
        "--arquivo",
        pdf.caminho(),
        "--tipo",
        "Anexo",
    ]);
    assert_eq!(saida.status.code(), Some(0), "{:?}", saida.stderr);
    assert_eq!(saida.stderr, b"");
    let humano = String::from_utf8(saida.stdout).unwrap();
    assert_eq!(
        humano,
        format!(
            "IES: arquivo local {} (23 bytes, SHA-256 49286c45c90723857946342e0610a2b28974f176218f77f79efacc34cc4567bc) inspecionado para 23106.000001/2026-11. Tipo solicitado: Anexo. Nenhuma tela do SEI foi consultada; não é prévia remota e nenhuma escrita está habilitada.\n",
            pdf.0.file_name().unwrap().to_str().unwrap()
        )
    );
}
