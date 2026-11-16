//! Ensaio com cassette original: IES interpreta, SQLite guarda,
//! anti-IES serve pelo socket, o mesmo leitor da CLI compara.
use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
use crate::tela_html::HtmlServido;
use iesei::banco_cassettes::{ControleResumido, Linha, guardar, iniciar};
use iesei::observacao::{FaixaObservada, TabelaObservada, UnidadeExibida};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn linhas_da_cassette(
    tabela: &crate::dominio::Tabela<crate::dominio::Processo>,
    faixa: crate::paginas::FaixaObservada,
) -> TabelaObservada<Linha> {
    let faixa = match faixa {
        crate::paginas::FaixaObservada::SemLinhas => None,
        crate::paginas::FaixaObservada::Lidas { inicio, fim } => {
            Some(FaixaObservada::nova(inicio, fim).unwrap())
        }
    };
    TabelaObservada::nova(
        Some(tabela.total_declarado),
        faixa,
        tabela
            .processos
            .iter()
            .map(|p| Linha {
                id: p.id.as_str().to_owned(),
                numero: p.numero.clone(),
                visualizado: p.visualizado,
                sigiloso: p.sigiloso,
            })
            .collect(),
    )
    .unwrap()
}

/// A cassette original (não uma fixture criada pelo anti-IES) fornece o
/// resultado independente; nenhuma rede SEI, login ou POST é executado.
#[test]
#[ignore = "cassette pública SHA-verificada de UnB no IES_ACERVO_CACHE; offline"]
fn cassette_unb_controle_r_fecha_ciclo_http_e_leitor_da_cli() {
    let card = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("cassettes/unb/unidade-processos-paginado-unb--e02dbcfde8c4.md");
    let cache = match std::env::var_os("IES_ACERVO_CACHE") {
        Some(valor) => std::path::PathBuf::from(valor),
        None => std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/ies"),
    };
    let zip = ZipDoAcervo::ler(&cache.join("todos-sei-cassettes-unb.zip")).unwrap();
    let captura = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
    let instalacao = crate::cassette::acervo::gravacao::instalacao(&captura.json)
        .expect("instalação da cassette conferida");
    let passos = captura.json["interactions"].as_array().unwrap();
    let passo = passos
        .iter()
        .find(|it| {
            it["request"]["method"] == "GET"
                && it["request"]["operation"] == "procedimento_controlar"
                && it["response"]["status_code"] == 200
        })
        .expect("GET Controle publicado pela cassette");
    let original = HtmlServido::de_teste(&html_da_resposta(passo).unwrap());
    assert!(!crate::controle_reconhecido::publica_visao_detalhada(
        &original
    ));
    let esperado = crate::paginas::juntar(std::slice::from_ref(&original), None).unwrap();
    let unidade = crate::unidade_atual_cli::unidade_validada(&original).unwrap();
    assert_eq!(esperado.unidade.as_deref(), Some(unidade.as_str()));

    let snapshot = ControleResumido {
        instalacao: instalacao.clone(),
        unidade: UnidadeExibida::nova(&unidade).unwrap(),
        recebidos: linhas_da_cassette(
            &esperado.caixa.recebidos,
            esperado.recebidos.faixa_observada().unwrap(),
        ),
        gerados: linhas_da_cassette(
            &esperado.caixa.gerados,
            esperado.gerados.faixa_observada().unwrap(),
        ),
    };

    let pasta = std::env::temp_dir().join(format!(
        "anti-ies-ciclo-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&pasta).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&pasta, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let caminho = pasta.join("teste.sqlite3");
    let mut banco = rusqlite::Connection::open(&caminho).unwrap();
    iniciar(&banco).unwrap();
    guardar(&mut banco, "unidade-processos-paginado-unb", &snapshot).unwrap();
    drop(banco);

    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let banco = rusqlite::Connection::open_with_flags(
            &caminho,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .unwrap();
        let (mut fluxo, _) = escuta.accept().unwrap();
        anti_ies::atender_http(
            &mut fluxo,
            Some((&banco, &instalacao, "unidade-processos-paginado-unb")),
        )
        .unwrap();
    });
    let mut cliente = TcpStream::connect(endereco).unwrap();
    write!(
        cliente,
        "GET /sei/controlador.php?acao=procedimento_controlar HTTP/1.1\r\nHost: localhost\r\n\r\n"
    )
    .unwrap();
    cliente.shutdown(std::net::Shutdown::Write).unwrap();
    let mut resposta = String::new();
    cliente.read_to_string(&mut resposta).unwrap();
    servidor.join().unwrap();
    let (cabecalho, corpo) = resposta.split_once("\r\n\r\n").unwrap();
    assert!(cabecalho.starts_with("HTTP/1.1 200 OK"), "{cabecalho}");
    assert!(cabecalho.contains("Content-Type: text/html; charset=utf-8"));

    let corpo = HtmlServido::de_teste(corpo);
    let obtido = crate::paginas::juntar(std::slice::from_ref(&corpo), None).unwrap();
    let unidade_anties = crate::unidade_atual_cli::unidade_validada(&corpo).unwrap();
    assert_eq!(unidade_anties, unidade);
    assert_eq!(obtido.unidade, esperado.unidade);
    assert_eq!(obtido.caixa, esperado.caixa);
    assert_eq!(
        obtido.recebidos.faixa_observada().unwrap(),
        esperado.recebidos.faixa_observada().unwrap()
    );
    assert_eq!(
        obtido.gerados.faixa_observada().unwrap(),
        esperado.gerados.faixa_observada().unwrap()
    );
    std::fs::remove_dir_all(&pasta).unwrap();
}
