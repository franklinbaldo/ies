//! #332: comando externo inspeciona PDF offline e recusa escrita sem sessão.
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

fn json(saida: &Output) -> serde_json::Value {
    serde_json::from_slice(&saida.stdout).expect("saída deve ser JSON")
}

struct PdfTeste(PathBuf);

impl PdfTeste {
    fn novo() -> Self {
        let nome = format!(
            "ies-332-cli-{}-{}.pdf",
            std::process::id(),
            PROXIMO.fetch_add(1, Ordering::Relaxed)
        );
        let caminho = std::env::temp_dir().join(nome);
        fs::write(&caminho, b"%PDF-1.4\n1 0 obj\n%%EOF\n").unwrap();
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
fn pdf_regular_publica_inspecao_local_sem_sessao_ou_http() {
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
    assert_eq!(valor["arquivo"]["tamanho_bytes"], 23);
    assert_eq!(
        valor["arquivo"]["sha256"],
        "49286c45c90723857946342e0610a2b28974f176218f77f79efacc34cc4567bc"
    );
    assert_eq!(valor["previa_remota"], false);
    assert_eq!(valor["requisicoes"], 0);
    assert_eq!(valor["escrita_remota"], false);
}

#[test]
fn confirmar_recusa_antes_de_ler_sessao_inexistente() {
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
    assert_eq!(saida.status.code(), Some(1));
    assert_eq!(json(&saida)["causa"], "CadastroNaoValidado");
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
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(json(&saida)["causa"], serde_json::json!({"Pdf": "Ausente"}));
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
    assert_eq!(saida.status.code(), Some(2));
    assert_eq!(
        json(&saida)["causa"],
        serde_json::json!({"Pdf": "NomeInvalido"})
    );
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
    assert_eq!(
        json(&saida)["causa"],
        serde_json::json!({"Tipo": "Controle"})
    );
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
    assert_eq!(json(&saida)["causa"], serde_json::json!({"Tipo": "Vazio"}));
    assert!(json(&saida).get("subcodigo").is_none());
}

#[test]
fn numero_invalido_expoe_tipo_de_causa_sem_subcodigo() {
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
    let valor = json(&saida);
    assert_eq!(valor["causa"], serde_json::json!({"Processo": "Vazio"}));
    assert!(valor.get("subcodigo").is_none());
}
