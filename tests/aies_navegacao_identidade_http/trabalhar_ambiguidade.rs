//! #60 / BDD-60-05: identidade repetida não deve mudar a rota selecionada.
//! A cassette UnB observa procedimento_trabalhar, mas não esta duplicação;
//! as recusas são hipóteses sintéticas com SQLite efêmero.
use super::*;

#[test]
fn http_trabalhar_nao_cai_na_rota_de_processo_quando_documento_e_repetido() {
    let servidor = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = servidor.local_addr().unwrap();
    let atendimento = thread::spawn(move || {
        let (banco, origem) = estado();
        let antes: Vec<_> = ["42", "43"]
            .iter()
            .map(|id| documento_versoes::ler(&banco, &origem, CENARIO, id).unwrap())
            .collect();
        for _ in 0..6 {
            let (mut tcp, cliente) = servidor.accept().unwrap();
            assert!(cliente.ip().is_loopback());
            atender_http(&mut tcp, Some((&banco, &origem, CENARIO))).unwrap();
        }
        let depois: Vec<_> = ["42", "43"]
            .iter()
            .map(|id| documento_versoes::ler(&banco, &origem, CENARIO, id).unwrap())
            .collect();
        assert_eq!(depois, antes, "leituras e recusas preservam o SQLite");
    });

    let primeiro = pedir_rota(
        endereco,
        "GET",
        "procedimento_trabalhar",
        "id_documento=42&id_procedimento=7",
    );
    assert!(primeiro.starts_with("HTTP/1.1 200 OK\r\n"), "{primeiro}");
    assert!(primeiro.contains("id_documento=42"), "{primeiro}");

    for parametros in [
        "id_documento=42&id_documento=42&id_procedimento=7",
        "id_documento=42&id_documento=43&id_procedimento=7",
        "id_documento=42&id_documento=42&id_protocolo=43&id_procedimento=7",
    ] {
        let resposta = pedir_rota(endereco, "GET", "procedimento_trabalhar", parametros);
        assert!(
            resposta.starts_with("HTTP/1.1 400 Bad Request\r\n"),
            "a identidade inválida não pode virar rota de processo: {parametros}: {resposta}"
        );
        assert!(!resposta.contains("id_documento=42"), "{resposta}");
        assert!(!resposta.contains("id_documento=43"), "{resposta}");
    }

    let segundo = pedir_rota(
        endereco,
        "GET",
        "procedimento_trabalhar",
        "id_documento=43&id_procedimento=7",
    );
    assert!(segundo.starts_with("HTTP/1.1 200 OK\r\n"), "{segundo}");
    assert!(segundo.contains("id_documento=43"), "{segundo}");
    assert!(!segundo.contains("id_documento=42"), "{segundo}");

    let metodo = pedir_rota(
        endereco,
        "POST",
        "procedimento_trabalhar",
        "id_documento=42&id_procedimento=7",
    );
    assert!(
        metodo.starts_with("HTTP/1.1 405 Method Not Allowed\r\n"),
        "{metodo}"
    );
    atendimento.join().unwrap();
}
