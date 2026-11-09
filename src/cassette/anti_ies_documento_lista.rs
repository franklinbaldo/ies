//! BDD-60-05: a oferta da lista de versões antes/depois da conclusão
//! é confrontada com a barra original `UnB`, não apenas com a CLI sintética.
//! A relação entre abertura por unidade e oferta é hipótese do anti-IES.
use crate::arvore_processo::{ler_arvore, ler_barra_do_documento, nup_declarado_na_arvore};
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
use crate::documento::NumeroDocumento;
use crate::documento_versoes::{VersaoDocumento, interpretar_historico};
use iesei::banco_cassettes::{self, ControleResumido, Linha, iniciar};
use iesei::documento_versoes::{DocumentoSimulado, Revisao, guardar};
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use iesei::processos::{self, ProcessoSimulado};
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::num::NonZeroU32;

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

fn barras_observadas(
    respostas: Vec<gravacao::Resposta<'_>>,
    instalacao: &iesei::instalacao::Instalacao,
    id: &str,
    numero: &str,
) -> Vec<(usize, String, String, bool)> {
    let mut barras = Vec::new();
    for resposta in respostas {
        if resposta.operacao != "procedimento_visualizar"
            || resposta.metodo != "GET"
            || resposta.status != 200
        {
            continue;
        }
        let Some(processo) = gravacao::id_procedimento(resposta.item) else {
            continue;
        };
        let Ok(html) = html_da_resposta(resposta.item) else {
            continue;
        };
        let Some(nup) = nup_declarado_na_arvore(&html) else {
            continue;
        };
        let Ok(arvore) = ler_arvore(&html, &nup, &processo) else {
            continue;
        };
        if !arvore
            .documentos
            .iter()
            .any(|d| d.id == id && d.numero_sei.as_deref() == Some(numero))
        {
            continue;
        }
        let barra = ler_barra_do_documento(instalacao, &html, id, &processo).unwrap();
        barras.push((resposta.passo, processo, nup, barra.versoes.is_some()));
    }
    barras
}

fn servir_barra(
    documento: DocumentoSimulado,
    instalacao: iesei::instalacao::Instalacao,
    id_processo: String,
    nup: String,
    revisoes_esperadas: usize,
) -> (SocketAddr, std::thread::JoinHandle<()>) {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let tarefa = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        guardar(&mut banco, "cassette-lista", &documento).unwrap();
        banco_cassettes::guardar(
            &mut banco,
            "cassette-lista",
            &ControleResumido {
                instalacao: instalacao.clone(),
                unidade: UnidadeExibida::nova("TESTE").unwrap(),
                recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
                gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            },
        )
        .unwrap();
        let mut processo = ProcessoSimulado {
            id: id_processo,
            numero: nup,
            unidade_origem: "TESTE".to_owned(),
            aberto_em: vec!["TESTE".to_owned()],
            anotacao_origem: None,
            destinos: vec![],
        };
        processos::guardar(&mut banco, &instalacao, "cassette-lista", &processo).unwrap();
        for passo in 0..4 {
            let (mut fluxo, _) = escuta.accept().unwrap();
            anti_ies::atender_http(&mut fluxo, Some((&banco, &instalacao, "cassette-lista")))
                .unwrap();
            if passo == 1 {
                processo.aberto_em.clear();
                processos::guardar(&mut banco, &instalacao, "cassette-lista", &processo).unwrap();
            }
        }
        assert_eq!(
            processos::ler(&banco, &instalacao, "cassette-lista", &processo.id)
                .unwrap()
                .unwrap()
                .aberto_em,
            Vec::<String>::new()
        );
        assert_eq!(
            banco
                .query_row("SELECT count(*) FROM documento_revisao", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            i64::try_from(revisoes_esperadas).unwrap()
        );
    });
    (endereco, tarefa)
}

#[test]
#[ignore = "ZIP UnB verificado no IES_ACERVO_CACHE; confronto offline no CI"]
fn cassette_unb_arvore_concluida_retira_link_de_versoes_no_http() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let card = raiz.join("cassettes/unb/liberacoes-e-versoes-leitura-unb--a781b0fbca8f.md");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/ies"),
        std::path::PathBuf::from,
    );
    let zip = ZipDoAcervo::ler(&cache.join("lote-9497057d512230ff.zip")).unwrap();
    let captura = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
    let instalacao = gravacao::instalacao(&captura.json).unwrap();
    let respostas = gravacao::respostas(&captura.json);
    let historico = respostas
        .iter()
        .find(|r| r.operacao == "documento_versao_listar" && r.metodo == "GET" && r.status == 200)
        .expect("histórico original UnB");
    let ids = gravacao::parametro(historico.item, "id_documento");
    let id = ids.first().expect("id_documento do histórico").to_string();
    assert!(ids.iter().all(|valor| *valor == id.as_str()));
    let original = respostas
        .iter()
        .find(|r| {
            r.operacao == "documento_visualizar"
                && r.metodo == "GET"
                && r.status == 200
                && gravacao::parametro(r.item, "id_documento").contains(&id.as_str())
        })
        .expect("conteúdo da mesma identidade");
    let html_original = html_da_resposta(original.item).unwrap();
    let identidade = crate::documento_web::identidade_do_titulo(&html_original).unwrap();
    let texto = crate::documento_web::texto_do_corpo(&html_original).unwrap();
    let conclusao = respostas
        .iter()
        .find(|r| r.operacao == "procedimento_concluir")
        .expect("conclusão registrada")
        .passo;

    let html_historico = html_da_resposta(historico.item).unwrap();
    let barras = barras_observadas(respostas, &instalacao, &id, &identidade.numero);
    let antes = barras
        .iter()
        .find(|(passo, _, _, oferecida)| *passo < conclusao && *oferecida)
        .expect("antes da conclusão, link observado na barra");
    let depois = barras
        .iter()
        .find(|(passo, processo, nup, oferecida)| {
            *passo > conclusao && !*oferecida && processo == &antes.1 && nup == &antes.2
        })
        .expect("após conclusão, mesma identidade sem link de versões");
    assert!(antes.0 < conclusao && depois.0 > conclusao);

    // Dados das linhas vêm do GET original do histórico, não de uma fixture.
    // Corpos das versões antigas não foram capturados: a simulação não os
    // apresenta neste confronto, que examina somente a lista de metadados.
    let numero = NumeroDocumento::novo(&identidade.numero).unwrap();
    let original_historico = interpretar_historico(&html_historico, &numero, &id, None).unwrap();
    let ultima = original_historico.atual().unwrap().numero();
    let revisoes: Vec<_> = (1..=ultima)
        .filter_map(|n| {
            let entrada = original_historico
                .selecionar(VersaoDocumento::nova(n).unwrap())
                .ok()?;
            Some(Revisao {
                versao: NonZeroU32::new(n).unwrap(),
                usuario: entrada.usuario.clone(),
                unidade: entrada.unidade.clone(),
                modificacao: entrada.modificacao.clone(),
                html_corpo: if n == ultima {
                    format!(
                        "<p>{}</p>",
                        texto
                            .replace('&', "&amp;")
                            .replace('<', "&lt;")
                            .replace('>', "&gt;")
                    )
                } else {
                    // Campo obrigatório no modelo: placeholder estrutural,
                    // não o corpo histórico que a cassette não registrou.
                    "<p></p>".to_owned()
                },
            })
        })
        .collect();
    assert_eq!(revisoes.len(), original_historico.len());

    let documento = DocumentoSimulado {
        instalacao: instalacao.clone(),
        id_documento: id.clone(),
        id_procedimento: antes.1.clone(),
        numero_processo: antes.2.clone(),
        numero: identidade.numero.clone(),
        tipo: identidade.tipo,
        revisoes,
    };
    let revisoes_esperadas = documento.revisoes.len();
    let (endereco, tarefa) = servir_barra(
        documento,
        instalacao.clone(),
        antes.1.clone(),
        antes.2.clone(),
        revisoes_esperadas,
    );
    let alvo = format!(
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento={}&id_documento={id}",
        antes.1
    );
    for esperado in [true, false] {
        let http = pedir(endereco, "GET", &alvo);
        let (cabecalho, html) = http.split_once("\r\n\r\n").unwrap();
        assert!(cabecalho.starts_with("HTTP/1.1 200 OK"), "{cabecalho}");
        let arvore = ler_arvore(html, &antes.2, &antes.1).unwrap();
        assert!(arvore.documentos.iter().any(|doc| doc.id == id));
        let barra = ler_barra_do_documento(&instalacao, html, &id, &antes.1).unwrap();
        assert_eq!(
            barra.versoes.is_some(),
            esperado,
            "barra original versus HTTP simulado"
        );
        if esperado {
            // Segundo GET da mesma sessão: confronto INDEPENDENTE dos
            // metadados lidos na cassette e dos renderizados do SQLite.
            let alvo_historico = format!(
                "/sei/controlador.php?acao=documento_versao_listar&id_documento={id}&id_procedimento={}",
                antes.1
            );
            let http_historico = pedir(endereco, "GET", &alvo_historico);
            let (cabecalho, html) = http_historico.split_once("\r\n\r\n").unwrap();
            assert!(cabecalho.starts_with("HTTP/1.1 200 OK"), "{cabecalho}");
            let simulado = interpretar_historico(html, &numero, &id, Some(revisoes_esperadas))
                .expect("leitor real do IES no histórico HTTP simulado");
            assert_eq!(simulado.len(), original_historico.len());
            assert_eq!(simulado.atual(), original_historico.atual());
            for n in 1..=ultima {
                let versao = VersaoDocumento::nova(n).unwrap();
                let Ok(observada) = original_historico.selecionar(versao) else {
                    assert!(simulado.selecionar(versao).is_err());
                    continue;
                };
                let projetada = simulado.selecionar(versao).unwrap();
                assert_eq!(
                    (
                        &projetada.usuario,
                        &projetada.unidade,
                        &projetada.modificacao
                    ),
                    (
                        &observada.usuario,
                        &observada.unidade,
                        &observada.modificacao
                    )
                );
            }
        }
    }
    assert!(pedir(endereco, "POST", &alvo).starts_with("HTTP/1.1 405"));
    tarefa.join().unwrap();
}
