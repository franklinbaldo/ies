//! BDD-60-05: a CLI Rust mantém a identidade do PDF antes/depois da anexação.
//! A variação com PASTA/AGUARDE tem prova HTTP separada; aqui a árvore é completa.
//! Hipótese sintética, não reprodução da transição em cassettes UnB/GO.
use super::{ANEXADO, CENARIO, Modo, PRINCIPAL, Pasta, cli};
use iesei::instalacao::Instalacao;
use iesei::pdf_externo::{self, Processo, TipoExterno};
use rusqlite::Connection;
use std::fs;
use std::path::Path;
use std::process::Output;

const PDF_SINTETICO: &[u8] = b"%PDF-1.4\n1 0 obj\n%%EOF\n";

pub(super) fn preparar(banco: &Connection, instalacao: &Instalacao) {
    pdf_externo::semear(
        banco,
        instalacao,
        CENARIO,
        &Processo {
            id: "10".into(),
            nup: PRINCIPAL.into(),
            unidade: "TESTE".into(),
        },
        &TipoExterno {
            id: "293".into(),
            nome: "Anexo".into(),
            limite_bytes: 100_000,
        },
    )
    .unwrap();
    let temporario = pdf_externo::receber_conteudo(
        banco,
        instalacao,
        CENARIO,
        "10",
        "identidade.pdf",
        PDF_SINTETICO,
    )
    .unwrap();
    pdf_externo::cadastrar(banco, instalacao, CENARIO, "10", "293", temporario.id).unwrap();
}

/// Executa o binário real em loopback, sempre sem cache do conteúdo.
/// `--saida` materializa a identidade pela foto do PDF e bytes preservados.
pub(super) fn baixar(pasta: &Pasta, arquivos: [&Path; 4], porta: u16, etapa: &str) -> Vec<u8> {
    let destino = pasta.0.join(format!("pdf_{etapa}"));
    fs::create_dir(&destino).unwrap();
    let saida = cli(
        pasta,
        arquivos,
        porta,
        &[
            "--json",
            "documento",
            "050000001",
            "--processo",
            PRINCIPAL,
            "--saida",
            destino.to_str().unwrap(),
            "--sem-cache",
        ],
    );
    assert_eq!(
        saida.status.code(),
        Some(0),
        "leitura {etapa} do PDF: stdout={} stderr={}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    fs::read(destino.join("050000001.pdf")).unwrap()
}

pub(super) fn conferir(antes: &Output, depois: &Output, pdf_antes: &[u8], pdf_depois: &[u8]) {
    assert_eq!(
        antes.status.code(),
        Some(0),
        "processo antes: {}",
        String::from_utf8_lossy(&antes.stderr)
    );
    assert_eq!(
        depois.status.code(),
        Some(0),
        "processo depois: {}",
        String::from_utf8_lossy(&depois.stderr)
    );
    let inicial: serde_json::Value = serde_json::from_slice(&antes.stdout).unwrap();
    let final_: serde_json::Value = serde_json::from_slice(&depois.stdout).unwrap();
    assert_eq!(inicial["numero"], PRINCIPAL);
    assert_eq!(final_["numero"], PRINCIPAL);
    assert!(
        inicial["anexados"].as_array().is_none_or(Vec::is_empty),
        "{inicial}"
    );
    assert_eq!(final_["anexados"][0]["numero"], ANEXADO);
    assert_eq!(pdf_antes, PDF_SINTETICO);
    assert_eq!(pdf_depois, PDF_SINTETICO);
    assert_eq!(pdf_antes, pdf_depois, "anexação não substitui os bytes");
}

#[test]
fn bdd_60_05_pdf_nao_troca_identidade_apos_anexacao_na_mesma_arvore() {
    super::executar(Modo::PdfAposAnexacao);
}
