//! Fixture positiva do cliente, não uma coorte persistida nem replay externo.
use super::*;

pub(super) fn responder(tcp: &mut TcpStream, descricao: &str, observacao: &str, id: &str) {
    let cab = cabecalho(tcp);
    let cab = std::str::from_utf8(&cab).unwrap();
    assert!(cab.starts_with("POST /sei/controlador.php?acao=protocolo_pesquisar&"));
    let tamanho = cab
        .lines()
        .find_map(|linha| {
            let (nome, valor) = linha.split_once(':')?;
            nome.eq_ignore_ascii_case("content-length")
                .then(|| valor.trim().parse::<usize>().unwrap())
        })
        .unwrap();
    assert!(tamanho < 64_000);
    let mut corpo = vec![0; tamanho];
    tcp.read_exact(&mut corpo).unwrap();
    let corpo = std::str::from_utf8(&corpo).unwrap();
    let valores = |campo: &str| {
        corpo
            .split('&')
            .filter_map(|par| par.split_once('='))
            .filter_map(|(nome, valor)| (nome == campo).then_some(valor))
            .collect::<Vec<_>>()
    };
    assert_eq!(valores("rdoPesquisarEm"), vec!["P"]);
    assert_eq!(valores("txtProtocoloPesquisa"), vec![""]);
    assert_eq!(valores("hdnInicio"), vec!["0"]);
    assert!(matches!(valores("q").as_slice(), [] | [""]));
    for (campo, esperado) in [
        ("txtDescricaoPesquisa", descricao),
        ("txtObservacaoPesquisa", observacao),
    ] {
        let recebidos = valores(campo);
        if esperado.is_empty() {
            assert!(matches!(recebidos.as_slice(), [] | [""]));
        } else {
            assert_eq!(recebidos, vec![esperado]);
        }
    }
    let hash = "c".repeat(64);
    write!(
        tcp,
        "HTTP/1.1 302 Found\r\nLocation: controlador.php?acao=procedimento_trabalhar&acao_origem=protocolo_pesquisar&id_procedimento={id}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={hash}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
}

#[test]
fn cli_envia_criterios_textuais_independentes_sem_nup() {
    for (filtros, descricao, observacao, id) in [
        (
            vec!["--descricao", "Ação + &"],
            "A%C3%A7%C3%A3o+%2B+%26",
            "",
            "41",
        ),
        (vec!["--observacao", " outra é "], "", "+outra+%C3%A9+", "42"),
        (
            vec!["--descricao", "Ação + &", "--observacao", " outra é "],
            "A%C3%A7%C3%A3o+%2B+%26",
            "+outra+%C3%A9+",
            "43",
        ),
    ] {
        let pasta = Pasta::nova();
        let mut args = vec!["--json", "processo", "buscar"];
        args.extend(filtros);
        let saida = executar_com_estado(
            &pasta,
            &args,
            EstadoDeBusca::TextoProcessual(descricao, observacao, id),
        );
        assert_eq!(
            saida.status.code(),
            Some(0),
            "{args:?}: {}",
            String::from_utf8_lossy(&saida.stderr)
        );
        let json: Value = serde_json::from_slice(&saida.stdout).unwrap();
        assert_eq!(json["resultado"], "processo_unico");
        assert_eq!(json["termo"], "");
        assert_eq!(json["unidade"], "TESTE");
        assert_eq!(json["requisicoes"], 3);
        assert_eq!(json["processo"], serde_json::json!({"id": id}));
    }
}

#[test]
fn cli_texto_desconhecido_nao_herda_zero_do_periodo() {
    for filtro in ["--descricao", "--observacao"] {
        let pasta = Pasta::nova();
        let saida = executar_com_estado(
            &pasta,
            &[
                "--json",
                "processo",
                "buscar",
                "--de",
                "01/01/2026",
                "--ate",
                "31/01/2026",
                filtro,
                "Ação + &",
            ],
            EstadoDeBusca::PeriodoProcessual,
        );
        assert_eq!(saida.status.code(), Some(1), "{filtro}");
        assert!(saida.stdout.is_empty(), "{filtro}");
    }
}
