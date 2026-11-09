use super::*;

pub(super) fn aplicar_post_no_sqlite_por_http_local(endereco: std::net::SocketAddr, escolha: &str) {
    // Um ator independente modifica só o SQLite do anti-IES pela rota HTTP.
    // A CLI da etapa seguinte precisa reler o estado, sem executar --confirmar.
    let corpo =
        format!("hdnInfraTipoPagina=2&hdnIdProtocolo=10&selAtribuicao={escolha}&sbmSalvar=Salvar");
    let requisicao = format!(
        "POST /sei/controlador.php?acao=procedimento_atribuicao_cadastrar&id_procedimento=10 HTTP/1.1\r\nHost: sei.test\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{corpo}",
        corpo.len()
    );
    let mut canal = TcpStream::connect(endereco).unwrap();
    canal.write_all(requisicao.as_bytes()).unwrap();
    canal.shutdown(Shutdown::Write).unwrap();
    let mut resposta = Vec::new();
    canal.read_to_end(&mut resposta).unwrap();
    assert!(
        resposta.starts_with(b"HTTP/1.1 302 "),
        "o POST local independente precisa receber um 302"
    );
}

#[test]
fn previa_remover_reflete_atribuicao_aplicada_antes_por_post_local() {
    // A mutação preparatória é externa à CLI, mas apenas no SQLite efêmero.
    // O comando público continua sendo uma prévia sem escrita.
    let (saida, eventos, _, atribuicao_final) = executar_apos_post_local(true, true, Some("8"));
    assert_eq!(saida.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["usuario_atual"], "aluno2");
    assert!(json["usuario_pedido"].is_null());
    assert_eq!(json["confirmado"], false);
    assert_eq!(eventos.len(), 5);
    assert_eq!(atribuicao_final, "aluno2");
}

#[test]
fn previa_atribuir_reflete_remocao_aplicada_antes_por_post_local() {
    let (saida, eventos, _, atribuicao_final) = executar_apos_post_local(true, false, Some("null"));
    assert_eq!(saida.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["usuario_atual"], "");
    assert_eq!(json["usuario_pedido"], "aluno2");
    assert_eq!(json["confirmado"], false);
    assert_eq!(eventos.len(), 5);
    assert_eq!(atribuicao_final, "");
}
