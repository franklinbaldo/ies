//! Segunda vertical: duas páginas mensais reais de retorno da `UnB`, sem SEI.
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
use iesei::banco_cassettes::iniciar;
use iesei::retornos_cassettes::{Mes, Retorno, Situacao, guardar};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn converter(retorno: &crate::unidade_retornos::Retorno) -> Retorno {
    let situacao = match retorno.situacao {
        "aguardando" => Situacao::Aguardando,
        "devolver" => Situacao::Devolver,
        _ => panic!("categoria não observada pelo IES"),
    };
    Retorno {
        situacao,
        numero: retorno.numero.clone(),
        unidade: retorno.unidade.clone(),
        data_programada: retorno.data_programada.clone(),
        // O armazenamento da cassette guarda célula vazia como NULL.
        retornado_em: match &retorno.retornado_em {
            crate::unidade_retornos::DataDeRetorno::Pendente => None,
            crate::unidade_retornos::DataDeRetorno::Em(data) => Some(data.clone()),
        },
        prazo_restante: match &retorno.prazo_restante {
            crate::unidade_retornos::PrazoRestante::SemPrazo => None,
            crate::unidade_retornos::PrazoRestante::Em(prazo) => Some(prazo.clone()),
        },
    }
}

#[test]
#[ignore = "cassette original UnB e ZIP SHA-verificado em IES_ACERVO_CACHE; offline"]
fn cassette_unb_retornos_mensais_fecha_ciclo_http_e_leitor() {
    let card = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("cassettes/unb/unidade-listar-retornos-unb--61ae9a77b5df.md");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/ies"),
        std::path::PathBuf::from,
    );
    let zip = ZipDoAcervo::ler(&cache.join("todos-sei-cassettes-unb.zip")).unwrap();
    let captura = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
    let instalacao = gravacao::instalacao(&captura.json).unwrap();
    let mut meses = Vec::new();

    let pasta = std::env::temp_dir().join(format!(
        "anti-ies-retornos-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&pasta).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&pasta, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let caminho = pasta.join("retornos.sqlite3");
    let mut banco = rusqlite::Connection::open(&caminho).unwrap();
    iniciar(&banco).unwrap();

    for resposta in gravacao::respostas(&captura.json) {
        if resposta.operacao != "retorno_programado_listar"
            || resposta.metodo != "GET"
            || resposta.status != 200
        {
            continue;
        }
        let datas = gravacao::parametro(resposta.interacao, "data_inicial");
        let Some(inicio) = datas.first().copied() else {
            continue; // Calendário, não página mensal.
        };
        let finais = gravacao::parametro(resposta.interacao, "data_final");
        let fim = finais
            .first()
            .copied()
            .expect("mês da cassette tem data final");
        let html = html_da_resposta(resposta.interacao).unwrap();
        let esperado = crate::unidade_retornos::extrair_retornos(&html).unwrap();
        assert_eq!(esperado.len(), 2, "não reduzir o conteúdo da cassette");
        let mes = Mes {
            instalacao: instalacao.clone(),
            data_inicial: inicio.to_owned(),
            data_final: fim.to_owned(),
            itens: esperado.iter().map(converter).collect(),
        };
        guardar(&mut banco, "unidade-listar-retornos-unb", &mes).unwrap();
        meses.push((mes, esperado));
    }
    assert_eq!(meses.len(), 2, "as duas páginas mensais devem ser lidas");
    drop(banco);

    for (mes, esperado) in meses {
        let servidor = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let endereco = servidor.local_addr().unwrap();
        let caminho = caminho.clone();
        let instalacao = instalacao.clone();
        let tarefa = std::thread::spawn(move || {
            let banco = rusqlite::Connection::open_with_flags(
                &caminho,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                    | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
            )
            .unwrap();
            let (mut fluxo, _) = servidor.accept().unwrap();
            anti_ies::atender_http(
                &mut fluxo,
                Some((&banco, &instalacao, "unidade-listar-retornos-unb")),
            )
            .unwrap();
        });
        let mut cliente = TcpStream::connect(endereco).unwrap();
        write!(
            cliente,
            "GET /sei/controlador.php?acao=retorno_programado_listar&data_inicial={}&data_final={} HTTP/1.1\r\nHost: localhost\r\n\r\n",
            mes.data_inicial, mes.data_final
        )
        .unwrap();
        cliente.shutdown(std::net::Shutdown::Write).unwrap();
        let mut resposta = String::new();
        cliente.read_to_string(&mut resposta).unwrap();
        tarefa.join().unwrap();
        let (cabecalho, corpo) = resposta.split_once("\r\n\r\n").unwrap();
        assert!(cabecalho.starts_with("HTTP/1.1 200 OK"));
        assert!(cabecalho.contains("Content-Type: text/html; charset=utf-8"));
        let obtido = crate::unidade_retornos::extrair_retornos(corpo).unwrap();
        assert_eq!(obtido, esperado, "mesmo significado da cassette original");
    }
    std::fs::remove_dir_all(&pasta).unwrap();
}
