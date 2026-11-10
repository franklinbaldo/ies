//! #60 / BDD-60-05: identidade de protocolo ambigua nao e campo ausente.
//! A cassette `UnB` observa `procedimento_trabalhar`, mas nao essas repeticoes:
//! as recusas deste teste sao hipoteses sinteticas, nao replay do SEI.
use super::*;

#[test]
fn http_trabalhar_recusa_protocolo_ou_procedimento_repetido_sem_fallback() {
    let servidor = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = servidor.local_addr().unwrap();
    let atendimento = thread::spawn(move || {
        let (banco, origem) = estado();
        let antes: Vec<_> = ["42", "43"]
            .iter()
            .map(|id| documento_versoes::ler(&banco, &origem, CENARIO, id).unwrap())
            .collect();
        for _ in 0..7 {
            let (mut tcp, cliente) = servidor.accept().unwrap();
            assert!(cliente.ip().is_loopback());
            atender_http(&mut tcp, Some((&banco, &origem, CENARIO))).unwrap();
        }
        let depois: Vec<_> = ["42", "43"]
            .iter()
            .map(|id| documento_versoes::ler(&banco, &origem, CENARIO, id).unwrap())
            .collect();
        assert_eq!(depois, antes, "recusas nao alteram nenhum documento");
    });

    for parametros in [
        "id_protocolo=42&id_protocolo=43",
        "id_protocolo=42&id_protocolo=42&id_procedimento=7",
        "id_protocolo=42&id_procedimento=7&id_procedimento=7",
        "id_protocolo=&id_procedimento=7",
    ] {
        let resposta = pedir_rota(endereco, "GET", "procedimento_trabalhar", parametros);
        assert!(
            resposta.starts_with("HTTP/1.1 400 Bad Request\r\n"),
            "identidade malformada nao pode virar rota alternativa: {parametros}: {resposta}"
        );
        assert!(!resposta.contains("0011519"), "{resposta}");
        assert!(!resposta.contains("0011520"), "{resposta}");
    }

    for (id, outro) in [("42", "43"), ("43", "42")] {
        let resposta = pedir_rota(
            endereco,
            "GET",
            "procedimento_trabalhar",
            &format!("id_documento={id}&id_procedimento=7"),
        );
        assert!(resposta.starts_with("HTTP/1.1 200 OK\r\n"), "{resposta}");
        assert!(
            resposta.contains(&format!("id_documento={id}")),
            "{resposta}"
        );
        assert!(
            !resposta.contains(&format!("id_documento={outro}")),
            "{resposta}"
        );
    }

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
