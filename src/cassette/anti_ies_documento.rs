//! Contraste parcial com conteúdo original da cassette `UnB` (#60).
//! O leitor do IES estabelece identidade/texto; o anti-IES projeta um
//! documento do SQLite e os mesmos leitores conferem a resposta HTTP.
//! Processo e revisão simulados não são evidência de paridade externa.
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
use iesei::banco_cassettes::iniciar;
use iesei::documento_versoes::{DocumentoSimulado, Revisao, guardar};
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::num::NonZeroU32;

include!("anti_ies_documento_fontes.rs");

fn escapar(texto: &str) -> String {
    texto
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn pedir(endereco: SocketAddr, metodo: &str, alvo: &str) -> String {
    let mut tcp = TcpStream::connect(endereco).unwrap();
    write!(
        tcp,
        "{metodo} {alvo} HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    tcp.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    tcp.read_to_string(&mut resposta).unwrap();
    resposta
}

/// Serve a revisão observada e confere que GET e POST recusado não a alteram.
fn servir_conteudo_observado(
    documento: DocumentoSimulado,
    instalacao: iesei::instalacao::Instalacao,
    requisicoes: usize,
) -> (SocketAddr, std::thread::JoinHandle<()>) {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let tarefa = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        guardar(&mut banco, "cassette-conteudo", &documento).unwrap();
        for _ in 0..requisicoes {
            let (mut fluxo, _) = escuta.accept().unwrap();
            anti_ies::atender_http(&mut fluxo, Some((&banco, &instalacao, "cassette-conteudo")))
                .unwrap();
        }
        assert_eq!(
            banco
                .query_row("SELECT count(*) FROM documento_revisao", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            1,
            "GET e POST recusado não modificam o SQLite"
        );
        assert_eq!(
            iesei::documento_versoes::ler(
                &banco,
                &instalacao,
                "cassette-conteudo",
                &documento.id_documento,
            )
            .unwrap(),
            Some(documento),
            "a navegação e as recusas preservam todo o documento",
        );
    });

    (endereco, tarefa)
}

/// Extrai só a região editável: o rodapé do anti-IES não é conteúdo do documento.
fn texto_do_editor_simulado(html: &str) -> String {
    let corpo = html
        .split_once("<main id='corpoEditavel'>")
        .unwrap()
        .1
        .split_once("</main>")
        .unwrap()
        .0;
    crate::documento_web::texto_do_corpo(&format!("<body>{corpo}</body>")).unwrap()
}

#[test]
#[ignore = "ZIP original UnB SHA-verificado no IES_ACERVO_CACHE; somente offline"]
fn cassette_unb_conteudo_documento_compara_leitores_por_http() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let card = raiz.join("cassettes/unb/liberacoes-e-versoes-leitura-unb--a781b0fbca8f.md");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/ies"),
        std::path::PathBuf::from,
    );
    let zip = ZipDoAcervo::ler(&cache.join("lote-9497057d512230ff.zip")).unwrap();
    let captura = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
    let instalacao = gravacao::instalacao(&captura.json).unwrap();
    let originais: Vec<_> = gravacao::respostas(&captura.json)
        .into_iter()
        .filter(|r| r.operacao == "documento_visualizar" && r.metodo == "GET" && r.status == 200)
        .collect();
    assert_eq!(
        originais.len(),
        1,
        "não inventar resposta ausente na cassette"
    );
    let ids = gravacao::parametro(originais[0].item, "id_documento");
    let id_observado = ids.first().copied().expect("id_documento observado no GET");
    assert!(
        ids.iter().all(|valor| *valor == id_observado),
        "o GET original deve apontar para um único documento"
    );
    let html_original = html_da_resposta(originais[0].item).unwrap();
    let identidade = crate::documento_web::identidade_do_titulo(&html_original).unwrap();
    let texto_original = crate::documento_web::texto_do_corpo(&html_original).unwrap();
    assert!(!texto_original.contains("infra_hash"));
    let (processo_observado, nup_observado) =
        processo_observado_da_arvore(&captura.json, id_observado, &identidade.numero);
    let revisao = revisao_do_conteudo_observado(
        &captura.json,
        id_observado,
        &identidade.numero,
        &html_original,
        &texto_original,
    );
    let numero_versao = revisao.versao.get();

    let documento = DocumentoSimulado {
        instalacao: instalacao.clone(),
        id_documento: id_observado.to_owned(),
        id_procedimento: processo_observado.clone(),
        numero_processo: nup_observado.clone(),
        numero: identidade.numero.clone(),
        tipo: identidade.tipo.clone(),
        revisoes: vec![revisao],
    };
    let (endereco, tarefa) = servir_conteudo_observado(documento, instalacao, 3);
    let url = format!(
        "/sei/controlador.php?acao=documento_visualizar&id_documento={id_observado}&versao={numero_versao}&id_procedimento={processo_observado}"
    );
    let resposta = pedir(endereco, "GET", &url);
    let (cabecalho, html_anti) = resposta.split_once("\r\n\r\n").unwrap();
    assert!(cabecalho.starts_with("HTTP/1.1 200 OK"), "{cabecalho}");
    let identidade_anti = crate::documento_web::identidade_do_titulo(html_anti).unwrap();
    assert_eq!(
        crate::rodape_do_documento::versao_do_rodape(html_anti)
            .map(crate::documento_versoes::VersaoDocumento::numero),
        Some(numero_versao),
        "o anti-IES precisa devolver a versão observada, não uma revisão suposta"
    );
    assert_eq!(
        identidade_anti, identidade,
        "número/tipo vêm da fonte original"
    );
    // O rodapé acrescentado pelo anti-IES não faz parte do documento.
    // Conferir a igualdade de TODO o texto projetado dentro do editor,
    // em vez de aceitar apenas um prefixo que esconderia truncamentos/anexos.
    let texto_anti = texto_do_editor_simulado(html_anti);
    assert_eq!(
        texto_anti, texto_original,
        "o texto da origem deve sobreviver integralmente à projeção"
    );
    let cruzado = pedir(
        endereco,
        "GET",
        &format!(
            "/sei/controlador.php?acao=documento_visualizar&id_documento={id_observado}&versao={numero_versao}&id_procedimento={processo_observado}0"
        ),
    );
    assert!(
        cruzado.starts_with("HTTP/1.1 404"),
        "identidade de outro processo"
    );
    assert!(pedir(endereco, "POST", &url).starts_with("HTTP/1.1 405"));
    tarefa.join().unwrap();
}

#[test]
#[ignore = "ZIP original UnB SHA-verificado no IES_ACERVO_CACHE; somente offline"]
fn cassette_unb_historico_preserva_versoes_e_metadados_por_http() {
    // Confronto separado: as versões, os usuários, unidades e datas vêm
    // da lista original. O corpo de versões não capturadas é hipótese
    // descartável exigida pelo modelo, NUNCA uma observação da cassette.
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let card = raiz.join("cassettes/unb/liberacoes-e-versoes-leitura-unb--a781b0fbca8f.md");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/ies"),
        std::path::PathBuf::from,
    );
    let zip = ZipDoAcervo::ler(&cache.join("lote-9497057d512230ff.zip")).unwrap();
    let captura = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
    let instalacao = gravacao::instalacao(&captura.json).unwrap();
    let originais: Vec<_> = gravacao::respostas(&captura.json)
        .into_iter()
        .filter(|r| r.operacao == "documento_versao_listar" && r.metodo == "GET" && r.status == 200)
        .collect();
    assert_eq!(originais.len(), 1, "usar somente o histórico gravado");
    let ids = gravacao::parametro(originais[0].item, "id_documento");
    let id = ids.first().copied().expect("id_documento na requisição");
    assert!(
        ids.iter().all(|valor| *valor == id),
        "a identidade da requisição deve ser inequívoca"
    );
    // A lista original não traz número SEI. Cruzamos o GET de conteúdo.
    let (numero_observado, tipo_observado) = identidade_do_conteudo_observado(&captura.json, id);
    let (processo_observado, nup_observado) =
        processo_observado_da_arvore(&captura.json, id, &numero_observado);
    let numero = crate::documento::NumeroDocumento::novo(&numero_observado).unwrap();
    let html_original = html_da_resposta(originais[0].item).unwrap();
    let historico_original =
        crate::documento_versoes::interpretar_historico(&html_original, &numero, id, None).unwrap();
    let atual = historico_original
        .atual()
        .expect("histórico não vazio")
        .numero();
    let (revisoes, versoes) = revisoes_da_cassette(&historico_original);
    assert!(
        revisoes.iter().all(|r| r.html_corpo == "<p></p>"),
        "o replay da lista não pode atribuir prosa inventada a versões sem corpo capturado"
    );

    let documento = DocumentoSimulado {
        instalacao: instalacao.clone(),
        id_documento: id.to_owned(),
        id_procedimento: processo_observado.clone(),
        numero_processo: nup_observado.clone(),
        numero: numero.as_str().to_owned(),
        tipo: tipo_observado,
        revisoes,
    };
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let quantidade_versoes = versoes.len();
    let tarefa = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        guardar(&mut banco, "cassette-historico", &documento).unwrap();
        for _ in 0..3 {
            let (mut fluxo, _) = escuta.accept().unwrap();
            anti_ies::atender_http(
                &mut fluxo,
                Some((&banco, &instalacao, "cassette-historico")),
            )
            .unwrap();
        }
        assert_eq!(
            banco
                .query_row("SELECT count(*) FROM documento_revisao", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            i64::try_from(quantidade_versoes).unwrap(),
            "leitura de histórico e recusas preservam o SQLite"
        );
    });

    let url = format!(
        "/sei/controlador.php?acao=documento_versao_listar&id_documento={id}&id_procedimento={processo_observado}"
    );
    let resposta = pedir(endereco, "GET", &url);
    let (cabecalho, html_anti) = resposta.split_once("\r\n\r\n").unwrap();
    assert!(cabecalho.starts_with("HTTP/1.1 200 OK"), "{cabecalho}");
    let historico_anti =
        crate::documento_versoes::interpretar_historico(html_anti, &numero, id, None).unwrap();
    assert_eq!(historico_anti.len(), historico_original.len());
    assert_eq!(historico_anti.atual(), historico_original.atual());
    for n in &versoes {
        let versao = crate::documento_versoes::VersaoDocumento::nova(*n).unwrap();
        let esperado = historico_original.selecionar(versao).unwrap();
        let observado = historico_anti.selecionar(versao).unwrap();
        assert_eq!(
            (
                &observado.usuario,
                &observado.unidade,
                &observado.modificacao
            ),
            (&esperado.usuario, &esperado.unidade, &esperado.modificacao),
            "metadados da versão {n} vêm da cassette, não da fixture"
        );
    }
    // Não se pode servir versão N inexistente como se fosse a mais recente.
    let ausente = atual.checked_add(1).unwrap();
    let visualizacao = format!(
        "/sei/controlador.php?acao=documento_visualizar&id_documento={id}&id_procedimento={processo_observado}&versao={ausente}"
    );
    assert!(pedir(endereco, "GET", &visualizacao).starts_with("HTTP/1.1 404"));
    assert!(pedir(endereco, "POST", &url).starts_with("HTTP/1.1 405"));
    tarefa.join().unwrap();
}

// Confronto separado da navegação de pesquisa pelo número SEI.
include!("anti_ies_documento_navegacao_unb.rs");


/// BDD-60-05: identidade documental sem '=' não é campo ausente.
/// Hipótese sintética: as cassettes publicam URLs bem formadas, não esta recusa.
#[test]
fn bdd60_http_identidade_documental_sem_igual_recusa_sem_fallback() {
    let instalacao =
        iesei::instalacao::Instalacao::interpretar("https://sei.exemplo/sei").unwrap();
    let documento = DocumentoSimulado {
        instalacao: instalacao.clone(),
        id_documento: "42".into(),
        id_procedimento: "10".into(),
        numero_processo: "00001.000001/2026-01".into(),
        numero: "0011519".into(),
        tipo: "Despacho".into(),
        revisoes: vec![Revisao {
            versao: NonZeroU32::new(1).unwrap(),
            usuario: "TESTE".into(),
            unidade: "ORIGEM".into(),
            modificacao: "10/11/2026".into(),
            html_corpo: "<p>Conteudo de teste</p>".into(),
        }],
    };
    let (endereco, tarefa) = servir_conteudo_observado(documento, instalacao, 4);
    let base = "/sei/controlador.php?acao=procedimento_trabalhar";
    let valido = format!("{base}&id_documento=42");
    assert!(
        pedir(endereco, "GET", &valido).starts_with("HTTP/1.1 200"),
        "identidade documental unica continua navegavel"
    );
    for alvo in [
        format!("{base}&id_documento=42&id_documento"),
        format!("{base}&id_protocolo=42&id_documento"),
    ] {
        let resposta = pedir(endereco, "GET", &alvo);
        assert!(
            resposta.starts_with("HTTP/1.1 400"),
            "campo id_documento sem '=' nao autoriza selecionar documento: {resposta}"
        );
        assert!(
            !resposta.contains("ifrArvore"),
            "recusa nao pode publicar arvore de outro caminho"
        );
    }
    assert!(
        pedir(endereco, "POST", &valido).starts_with("HTTP/1.1 405"),
        "leitura nao permite metodo mutador"
    );
    tarefa.join().unwrap(); // Guarda a comparacao integral da revisao no SQLite.
}
