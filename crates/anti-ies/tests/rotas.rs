use anti_ies::{Estado, despachar};

#[test]
fn saude_expoe_apenas_identidade_do_servidor_de_teste() {
    let resposta = despachar("GET", "/__anti_ies/health");
    assert_eq!(resposta.estado, Estado::Ok);
    assert_eq!(resposta.estado.codigo_http(), 200);
    assert_eq!(resposta.tipo_conteudo, "application/json; charset=utf-8");
    assert_eq!(resposta.corpo, r#"{"status":"ok","servidor":"anti-ies"}"#);
}

#[test]
fn nao_finge_sei_em_operacoes_ainda_nao_observadas() {
    for rota in [
        "/sei/controlador.php?acao=procedimento_trabalhar",
        "/controlador.php?acao=procedimento_trabalhar",
    ] {
        let resposta = despachar("GET", rota);
        assert_eq!(resposta.estado, Estado::NaoImplementado, "{rota}");
        assert_eq!(resposta.estado.codigo_http(), 501);
    }
}

#[test]
fn rota_de_saude_nao_aceita_escrita() {
    assert_eq!(
        despachar("POST", "/__anti_ies/health").estado,
        Estado::MetodoNaoPermitido
    );
}

#[test]
fn prefixo_parecido_nao_vira_controlador_sei() {
    assert_eq!(
        despachar("GET", "/sei/controlador.php.incorreto").estado,
        Estado::NaoEncontrado
    );
    assert_eq!(
        despachar("GET", "/__anti_ies/health?credencial=proibida").estado,
        Estado::NaoEncontrado
    );
}

#[test]
fn requisicao_inesperada_nao_expõe_estado_nem_sessao() {
    let resposta = despachar("GET", "/dados/privados");
    assert_eq!(resposta.estado, Estado::NaoEncontrado);
    assert!(!resposta.corpo.contains("Cookie"));
}
