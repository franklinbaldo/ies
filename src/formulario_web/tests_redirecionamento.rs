//! RED/GREEN do `Location` de escrita lido como dado: relativo (gravado
//! nas sete instalações) ou absoluto no host e na pasta do POST, `http://`
//! inclusive (medido pelo Todos no GO, issue #766). Hosts inventados.
use super::consulta_do_controlador;

const POST: &str =
    "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_enviar&infra_hash=1";
const CONSULTA: &str = "acao=arvore_visualizar&id_procedimento=10";

#[test]
fn relativo_e_absoluto_no_mesmo_host_e_pasta() {
    for localizacao in [
        format!("controlador.php?{CONSULTA}"),
        format!("http://sei.exemplo.gov.br/sei/controlador.php?{CONSULTA}"),
        format!("https://sei.exemplo.gov.br/sei/controlador.php?{CONSULTA}"),
        format!("HTTP://SEI.Exemplo.gov.br/sei/controlador.php?{CONSULTA}"),
    ] {
        assert_eq!(
            consulta_do_controlador(&localizacao, POST),
            Some(CONSULTA),
            "{localizacao}"
        );
    }
}

#[test]
fn outro_host_pasta_ou_forma_nao_e_o_controlador() {
    for localizacao in [
        format!("http://outro.exemplo.gov.br/sei/controlador.php?{CONSULTA}"),
        format!("https://sei.exemplo.gov.br.outro.net/sei/controlador.php?{CONSULTA}"),
        format!("http://sei.exemplo.gov.br:8080/sei/controlador.php?{CONSULTA}"),
        format!("http://x@sei.exemplo.gov.br/sei/controlador.php?{CONSULTA}"),
        format!("http://sei.exemplo.gov.br/sip/controlador.php?{CONSULTA}"),
        format!("http://sei.exemplo.gov.br/controlador.php?{CONSULTA}"),
        format!("http://sei.exemplo.gov.br/sei/../sei/controlador.php?{CONSULTA}"),
        format!("http://sei.exemplo.gov.br/sei/controlador_ajax.php?{CONSULTA}"),
        format!("//sei.exemplo.gov.br/sei/controlador.php?{CONSULTA}"),
        format!("ftp://sei.exemplo.gov.br/sei/controlador.php?{CONSULTA}"),
        format!("/sei/controlador.php?{CONSULTA}"),
        format!("http://sei.exemplo.gov.br/sei/controlador.php?{CONSULTA}#x"),
        format!("http://sei.exemplo.gov.br/sei/controlador.php?{CONSULTA} "),
        "http://sei.exemplo.gov.br/sei/controlador.php".to_owned(),
    ] {
        assert_eq!(
            consulta_do_controlador(&localizacao, POST),
            None,
            "{localizacao}"
        );
    }
    // Sem URL HTTPS do POST, nem a absoluta nem a relativa ficam valendo
    // contra outra pasta: a relativa não depende do POST.
    assert_eq!(
        consulta_do_controlador(
            &format!("http://sei.exemplo.gov.br/sei/controlador.php?{CONSULTA}"),
            "http://sei.exemplo.gov.br/sei/controlador.php"
        ),
        None
    );
}
