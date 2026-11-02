//! RED/GREEN do saneamento: texto só, espaços colapsados, segredos e URLs
//! redigidos por varredura explícita, corte na fronteira de caractere.
use super::{MAX_CARACTERES, sanear};

#[test]
fn controles_viram_espaco_e_espacos_colapsam() {
    assert_eq!(
        sanear("  Erro\u{0}\tno\r\n\n  banco\u{7f} de\u{a0}dados  ").as_deref(),
        Some("Erro no banco de dados")
    );
}

#[test]
fn texto_vazio_ou_so_espacos_nao_e_mensagem() {
    assert_eq!(sanear(""), None);
    assert_eq!(sanear(" \n\t\u{1} "), None);
}

#[test]
fn urls_viram_marcador_sem_vazar_a_assinatura() {
    let texto = "Falha em https://sei.go.gov.br/sei/controlador.php?acao=x&infra_hash=abc \
                 e HTTP://outro.gov.br/a?b=c.";
    assert_eq!(
        sanear(texto).as_deref(),
        Some("Falha em <url> e <url>"),
        "a URL vai até o próximo espaço"
    );
}

#[test]
fn valor_de_infra_hash_e_de_phpsessid_e_redigido() {
    let hash = "0123456789abcdef".repeat(4);
    let texto = format!("parametro infra_hash={hash}&x=1; cookie PHPSESSID=abc123; path=/");
    let saneado = sanear(&texto).unwrap();
    assert_eq!(
        saneado,
        "parametro infra_hash=<redigido>&x=1; cookie PHPSESSID=<redigido>; path=/"
    );
    assert!(!saneado.contains("abc123"));
    // O nome é procurado sem distinguir maiúsculas.
    assert_eq!(
        sanear("INFRA_HASH=zz phpsessid=qq").as_deref(),
        Some("INFRA_HASH=<redigido> phpsessid=<redigido>")
    );
}

#[test]
fn sequencia_de_32_ou_mais_hexadecimais_e_redigida() {
    let longo = "A1b2C3d4".repeat(4);
    assert_eq!(longo.len(), 32);
    let curto = &longo[..31];
    assert_eq!(
        sanear(&format!("token {longo} fim")).as_deref(),
        Some("token <hex> fim")
    );
    assert_eq!(
        sanear(&format!("token {curto} fim")),
        Some(format!("token {curto} fim"))
    );
    assert_eq!(
        sanear(&format!("id={longo}ff")).as_deref(),
        Some("id=<hex>")
    );
}

#[test]
fn corte_em_500_caracteres_na_fronteira_com_reticencias() {
    let texto = "çã".repeat(400);
    let saneado = sanear(&texto).unwrap();
    assert_eq!(saneado.chars().count(), MAX_CARACTERES);
    assert!(saneado.ends_with('…'));
    assert!(saneado.starts_with("çãçã"));
    let exato = "x".repeat(MAX_CARACTERES);
    assert_eq!(sanear(&exato), Some(exato.clone()), "no limite não corta");
}

#[test]
fn redacao_acontece_antes_do_corte() {
    let hash = "f".repeat(64);
    let texto = format!("{} infra_hash={hash}", "a ".repeat(240));
    let saneado = sanear(&texto).unwrap();
    assert!(!saneado.contains("fffff"), "{saneado}");
}
