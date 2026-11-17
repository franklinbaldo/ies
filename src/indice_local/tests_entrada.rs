//! A única URL sem assinatura que o IES monta, e só nessa forma.
use super::tests::instalacao;
use super::*;
use crate::instalacao::Instalacao;

#[test]
fn entrada_por_id_tem_exatamente_a_forma_medida() {
    assert_eq!(
        url_da_entrada_por_id(&instalacao(), "123").as_deref(),
        Ok(
            "https://sei.exemplo/sei/controlador.php?acao=procedimento_trabalhar&id_procedimento=123"
        )
    );
    let raiz = Instalacao::interpretar("https://sei.exemplo/").unwrap();
    assert_eq!(
        url_da_entrada_por_id(&raiz, "9").as_deref(),
        Ok("https://sei.exemplo/controlador.php?acao=procedimento_trabalhar&id_procedimento=9")
    );
}

#[test]
fn entrada_com_documento_tem_exatamente_a_forma_medida() {
    assert_eq!(
        url_da_entrada_por_id_com_documento(&instalacao(), "123", "456").as_deref(),
        Ok(
            "https://sei.exemplo/sei/controlador.php?acao=procedimento_trabalhar&id_procedimento=123&id_documento=456"
        )
    );
    for (processo, documento) in [("123", ""), ("", "456"), ("1", "4&x=1"), ("1", "4#")] {
        assert_eq!(
            url_da_entrada_por_id_com_documento(&instalacao(), processo, documento),
            Err(super::validacao::RegistroRecusado::IdProcedimentoInvalido),
            "{processo} {documento}"
        );
    }
}

#[test]
fn entrada_por_id_recusa_id_que_nao_e_digito() {
    for id in [
        "",
        "1&id_documento=2",
        "1#x",
        " 1",
        "1.0",
        "123456789012345678901",
        "１",
    ] {
        assert_eq!(
            url_da_entrada_por_id(&instalacao(), id),
            Err(super::validacao::RegistroRecusado::IdProcedimentoInvalido),
            "{id}"
        );
    }
}

/// Nenhum outro código do módulo monta `controlador.php?`: a entrada sem
/// assinatura é uma função só, e as URLs assinadas só chegam prontas.
#[test]
fn modulo_so_monta_uma_url_sem_assinatura() {
    let pasta = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/indice_local");
    let mut ocorrencias = Vec::new();
    for entrada in std::fs::read_dir(&pasta).unwrap() {
        let caminho = entrada.unwrap().path();
        let nome = caminho.file_name().unwrap().to_string_lossy().into_owned();
        if nome.starts_with("tests") {
            continue;
        }
        let texto = std::fs::read_to_string(&caminho).unwrap();
        let n = texto.matches("controlador.php?acao=").count();
        if n > 0 {
            ocorrencias.push((nome, n));
        }
    }
    let raiz = std::fs::read_to_string(pasta.with_extension("rs")).unwrap();
    assert!(!raiz.contains("controlador.php?acao="));
    assert_eq!(ocorrencias, [("entrada_por_id.rs".to_owned(), 2)]);
}
