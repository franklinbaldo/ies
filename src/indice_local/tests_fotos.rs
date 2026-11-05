//! Tabela `foto` (#207): o SHA-256 de `DOC.vN.html` por instalação,
//! número SEI e versão; a migração 2 → 3 preserva o resto.
use super::tests::{instalacao, pasta};
use super::*;
use crate::documento_versoes::VersaoDocumento;
use crate::instalacao::Instalacao;

fn v(n: u32) -> VersaoDocumento {
    VersaoDocumento::nova(n).unwrap()
}

#[test]
fn registra_e_le_o_sha_por_instalacao_numero_e_versao() {
    let raiz = pasta("fotos");
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO));
    let sha = "ab".repeat(32);
    assert_eq!(indice.foto(&instalacao(), "0011519", v(2)), None);
    indice
        .registrar_foto(&instalacao(), "0011519", v(2), &sha)
        .unwrap();
    assert_eq!(
        indice.foto(&instalacao(), "0011519", v(2)),
        Some(sha.clone())
    );
    assert_eq!(indice.foto(&instalacao(), "0011519", v(3)), None);
    assert_eq!(indice.foto(&instalacao(), "011519", v(2)), None);
    let outra = Instalacao::interpretar("https://outro.exemplo/sei").unwrap();
    assert_eq!(indice.foto(&outra, "0011519", v(2)), None);
    let novo = "cd".repeat(32);
    indice
        .registrar_foto(&instalacao(), "0011519", v(2), &novo)
        .unwrap();
    assert_eq!(indice.foto(&instalacao(), "0011519", v(2)), Some(novo));
    assert_eq!(
        indice.registrar_foto(&instalacao(), "0011519", v(2), "AB"),
        Err(RegistroRecusado::Sha256Invalido)
    );
}

#[test]
fn indice_desligado_nao_registra() {
    let indice = IndiceLocal::desligado();
    let sha = "ab".repeat(32);
    indice
        .registrar_foto(&instalacao(), "0011519", v(2), &sha)
        .unwrap();
    assert_eq!(indice.foto(&instalacao(), "0011519", v(2)), None);
}

#[test]
fn banco_na_versao_2_migra_para_3_sem_perder_documentos() {
    let raiz = pasta("fotos-migracao");
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    let indice = IndiceLocal::em(caminho.clone());
    indice
        .registrar_documentos(&instalacao(), "10", &[("0000101", "21")])
        .unwrap();
    let conexao = rusqlite::Connection::open(&caminho).unwrap();
    conexao
        .execute_batch("DROP TABLE foto; PRAGMA user_version = 2;")
        .unwrap();
    drop(conexao);
    let indice = IndiceLocal::em(caminho);
    assert!(
        indice
            .documento_por_numero(&instalacao(), "0000101")
            .is_some()
    );
    let sha = "ab".repeat(32);
    indice
        .registrar_foto(&instalacao(), "0000101", v(1), &sha)
        .unwrap();
    assert_eq!(indice.foto(&instalacao(), "0000101", v(1)), Some(sha));
}
