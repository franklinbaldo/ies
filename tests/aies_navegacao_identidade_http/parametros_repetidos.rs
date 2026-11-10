//! #60 / BDD-60-05: links de versão com `id_documento` duplicado.
//! A duplicação concordante vem da forma observada nos links de versões;
//! as contradições abaixo são hipóteses sintéticas, não capturas do SEI.
use super::*;

#[test]
fn http_url_de_versao_aceita_repeticao_observada_mas_recusa_identidade_ambigua() {
    let servidor = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = servidor.local_addr().unwrap();
    let atendimento = thread::spawn(move || {
        let (banco, origem) = estado();
        let antes: Vec<_> = ["42", "43"]
            .iter()
            .map(|id| documento_versoes::ler(&banco, &origem, CENARIO, id).unwrap())
            .collect();
        for _ in 0..8 {
            let (mut tcp, cliente) = servidor.accept().unwrap();
            assert!(cliente.ip().is_loopback());
            atender_http(&mut tcp, Some((&banco, &origem, CENARIO))).unwrap();
        }
        let depois: Vec<_> = ["42", "43"]
            .iter()
            .map(|id| documento_versoes::ler(&banco, &origem, CENARIO, id).unwrap())
            .collect();
        assert_eq!(depois, antes, "nenhuma recusa nem leitura muda as revisões");
    });

    // Não transformar parâmetros ambíguos em escolha arbitrária de corpo.
    for (acao, parametros) in [
        (
            "documento_visualizar",
            "id_documento=42&id_documento=43&id_procedimento=7",
        ),
        (
            "documento_versao_listar",
            "id_documento=42&id_documento=43&id_procedimento=7",
        ),
        (
            "documento_visualizar",
            "id_documento=42&id_protocolo=42&id_protocolo=42&id_procedimento=7",
        ),
        (
            "documento_versao_listar",
            "id_documento=42&id_procedimento=7&id_procedimento=7",
        ),
        (
            "documento_visualizar",
            "id_documento=42&id_procedimento=7&versao=1&versao=1",
        ),
    ] {
        let resposta = pedir_rota(endereco, "GET", acao, parametros);
        assert!(
            resposta.starts_with("HTTP/1.1 400 Bad Request\r\n"),
            "identidade ambígua: {acao} {parametros}: {resposta}"
        );
        assert!(!resposta.contains("Corpo do primeiro"), "{resposta}");
        assert!(!resposta.contains("Corpo do segundo"), "{resposta}");
        assert!(!resposta.contains("Lista de Versões"), "{resposta}");
    }

    // O SEI pode repetir *o mesmo* id_documento no link de uma versão.
    // Preservar o caminho positivo em vez de proibir a forma observada.
    let corpo = pedir_rota(
        endereco,
        "GET",
        "documento_visualizar",
        "id_documento=42&id_procedimento=7&versao=1&id_documento=42",
    );
    assert!(corpo.starts_with("HTTP/1.1 200 OK\r\n"), "{corpo}");
    assert!(corpo.contains("Corpo do primeiro"), "{corpo}");
    assert!(!corpo.contains("Corpo do segundo"), "{corpo}");

    let lista = pedir_rota(
        endereco,
        "GET",
        "documento_versao_listar",
        "id_documento=42&id_procedimento=7&id_documento=42",
    );
    assert!(lista.starts_with("HTTP/1.1 200 OK\r\n"), "{lista}");
    assert!(lista.contains("Lista de Versões"), "{lista}");

    let outro = pedir_rota(
        endereco,
        "GET",
        "documento_visualizar",
        "id_documento=43&id_procedimento=7",
    );
    assert!(outro.starts_with("HTTP/1.1 200 OK\r\n"), "{outro}");
    assert!(outro.contains("Corpo do segundo"), "{outro}");
    atendimento.join().unwrap();
}
