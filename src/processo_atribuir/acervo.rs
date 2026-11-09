//! Replay offline da atribuição (#264), sem enviar POST ou executar JS.
//! Confere ZIPs, membros, schema e host antes de usar o leitor real.
use crate::cassette::acervo::{INSTALACOES, Resultado, gravacao, publicar, tabela};
use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

struct Registro {
    leitor: &'static str,
    instalacao: String,
    resultado: Resultado,
}

/// Somente um ID oculto singular permite correlacionar a tela com seu
/// formulário; não usamos URL assinada de sessão como identidade pública.
fn id_da_tela(html: &str) -> Option<String> {
    let dom = tl::parse(html, tl::ParserOptions::default()).ok()?;
    let forms = nos_com_id(&dom, "frmAtividadeAtribuir").ok()?;
    let [form] = forms.as_slice() else {
        return None;
    };
    let parser = dom.parser();
    let form = form.get(parser)?.as_tag()?;
    let mut ids = Vec::new();
    for node in form.query_selector(parser, "input").into_iter().flatten() {
        let input = node.get(parser)?.as_tag()?;
        if valor_do_atributo(input, "name").ok()?.as_deref() == Some("hdnIdProtocolo") {
            ids.push(valor_do_atributo(input, "value").ok()?);
        }
    }
    let [Some(id)] = ids.as_slice() else {
        return None;
    };
    (!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).then(|| id.clone())
}

fn replay(instalacao: &str, json: &Value) -> Vec<Registro> {
    let mut registros = Vec::new();
    for resposta in gravacao::respostas(json) {
        if resposta.operacao != "procedimento_atribuicao_cadastrar" {
            continue;
        }
        let (leitor, resultado) = match (resposta.metodo, resposta.status) {
            ("POST", _) => (
                "resposta de escrita",
                Resultado::NaoAplicavel("post_nao_comprova_atribuicao"),
            ),
            ("GET", 200) => (
                "formulário de atribuição",
                match html_da_resposta(resposta.item) {
                    Err(_) => Resultado::NaoAplicavel("html_nao_observado"),
                    Ok(html) => {
                        let id =
                            gravacao::id_procedimento(resposta.item).or_else(|| id_da_tela(&html));
                        match id {
                            None => Resultado::NaoAplicavel("id_do_processo_nao_observado"),
                            Some(id) => match super::ler_tela(&html, &id) {
                                Ok(_) => Resultado::Lido,
                                Err(erro) => Resultado::Recusado(erro.subcodigo()),
                            },
                        }
                    }
                },
            ),
            _ => (
                "resposta sem formulário",
                Resultado::NaoAplicavel("status_ou_metodo_nao_lido"),
            ),
        };
        registros.push(Registro {
            leitor,
            instalacao: instalacao.to_owned(),
            resultado,
        });
    }
    registros
}

#[test]
fn replay_sintetico_nao_converte_post_em_prova_de_atribuicao() {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use serde_json::json;
    let html = r#"<form id="frmAtividadeAtribuir" method="post" action="controlador.php?acao=procedimento_atribuicao_cadastrar&id_procedimento=50693&infra_hash=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"><input type="hidden" name="hdnInfraTipoPagina" value="2"><button type="submit" name="sbmSalvar" value="Salvar">Salvar</button><select id="selAtribuicao" name="selAtribuicao"><option value="null" selected> </option><option value="1">aluno2 - Aluno Dois</option></select><input type="hidden" name="hdnIdProtocolo" value="50693"></form>"#;
    let json = json!({"interactions": [
        {
            "request": {"operation": "procedimento_atribuicao_cadastrar", "method": "GET"},
            "response": {"status_code": 200, "url": "https://sei.treinamento.unb.br/sei/controlador.php?acao=procedimento_atribuicao_cadastrar", "encoding": "utf-8", "headers": {"content-type": "text/html"}, "body_base64": STANDARD.encode(html)}
        },
        {
            "request": {"operation": "procedimento_atribuicao_cadastrar", "method": "POST"},
            "response": {"status_code": 302}
        }
    ]});
    let lidos = replay("unb", &json);
    assert_eq!(lidos.len(), 2);
    assert_eq!(lidos[0].resultado, Resultado::Lido);
    assert_eq!(
        lidos[1].resultado,
        Resultado::NaoAplicavel("post_nao_comprova_atribuicao")
    );
}

/// Compara o *mesmo* formulário observado em uma cassette pública com a
/// projeção de menu do anti-IES, usando o leitor real do IES nos dois HTMLs.
/// A unidade e o NUP são andaimes locais: o GET da atribuição não os publica.
/// Só os usuários, seus IDs, rótulos e o ID do formulário são confrontados.
/// Não se infere atribuição atual nem sucesso de POST a partir dessa prova.
fn confrontar_formularios(instalacao: &str, json: &Value) -> usize {
    use iesei::atribuicoes::{self, Observacao, Usuario};
    use iesei::banco_cassettes::{self, ControleResumido, Linha};
    use iesei::observacao::{TabelaObservada, UnidadeExibida};
    use iesei::processos::{self, ProcessoSimulado};
    use std::io::{Read as _, Write as _};
    use std::net::{Shutdown, TcpListener, TcpStream};

    const CENARIO: &str = "confronto-264-formulario-original";
    const UNIDADE_TECNICA: &str = "ACERVO";
    const NUP_TECNICO: &str = "00001.000001/2026-01";
    let origem = gravacao::instalacao(json).expect("origem verificada da cassette");
    let mut confrontados = 0;

    for resposta in gravacao::respostas(json) {
        if resposta.operacao != "procedimento_atribuicao_cadastrar"
            || resposta.metodo != "GET"
            || resposta.status != 200
        {
            continue;
        }
        let Ok(original) = html_da_resposta(resposta.item) else {
            continue;
        };
        let Some(id) = gravacao::id_procedimento(resposta.item).or_else(|| id_da_tela(&original))
        else {
            continue;
        };
        let Ok(esperado) = super::ler_tela(&original, &id) else {
            // O replay separado mantém a classificação de formas recusadas.
            continue;
        };
        let usuarios = esperado
            .usuarios
            .iter()
            .map(|u| {
                let (_, nome) = u
                    .rotulo
                    .split_once(" - ")
                    .expect("leitor aceitou rótulo login - nome");
                Usuario {
                    id: u.id.clone(),
                    login: u.login.clone(),
                    nome: nome.to_owned(),
                }
            })
            .collect();
        let observacao = Observacao {
            login_atual: String::new(),
            usuarios,
        };
        let mut banco = rusqlite::Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        banco_cassettes::guardar(
            &mut banco,
            CENARIO,
            &ControleResumido {
                instalacao: origem.clone(),
                unidade: UnidadeExibida::nova(UNIDADE_TECNICA).unwrap(),
                recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
                gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            },
        )
        .unwrap();
        processos::guardar(
            &mut banco,
            &origem,
            CENARIO,
            &ProcessoSimulado {
                id: id.clone(),
                numero: NUP_TECNICO.to_owned(),
                unidade_origem: UNIDADE_TECNICA.to_owned(),
                aberto_em: vec![UNIDADE_TECNICA.to_owned()],
                anotacao_origem: None,
                destinos: vec![],
            },
        )
        .unwrap();
        atribuicoes::registrar(
            &mut banco,
            &origem,
            CENARIO,
            &id,
            UNIDADE_TECNICA,
            &observacao,
        )
        .unwrap();

        let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let endereco = escuta.local_addr().unwrap();
        let servidor = std::thread::spawn(move || {
            let (mut fluxo, _) = escuta.accept().unwrap();
            anti_ies::atender_http(&mut fluxo, Some((&banco, &origem, CENARIO))).unwrap();
        });
        let mut cliente = TcpStream::connect(endereco).unwrap();
        write!(
            cliente,
            "GET /sei/controlador.php?acao=procedimento_atribuicao_cadastrar&id_procedimento={id} HTTP/1.1\r\nHost: localhost\r\n\r\n"
        )
        .unwrap();
        cliente.shutdown(Shutdown::Write).unwrap();
        let mut resposta_http = String::new();
        cliente.read_to_string(&mut resposta_http).unwrap();
        servidor.join().unwrap();
        let (cabecalho, corpo) = resposta_http
            .split_once("\r\n\r\n")
            .expect("resposta HTTP do anti-IES");
        assert!(
            cabecalho.starts_with("HTTP/1.1 200 OK"),
            "formulário observado não foi oferecido pelo anti-IES em {instalacao}"
        );
        let obtido = super::ler_tela(corpo, &id).expect("projeção HTTP do anti-IES");
        assert_eq!(
            obtido.usuarios, esperado.usuarios,
            "opções e rótulos divergem da cassette original em {instalacao}"
        );
        confrontados += 1;
    }
    confrontados
}

/// Rodado obrigatoriamente no CI após download dos ZIPs públicos com SHA-1.
/// O test harness normal não possui esses arquivos; não é WIP nem teste
/// desativado, e o job dedicado invoca este teste com --ignored.
#[test]
#[ignore = "acervo #264: executado obrigatoriamente no CI após conferir ZIPs; sem rede"]
fn replay_de_processo_atribuir_no_acervo() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("cassettes");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cache/ies"),
        PathBuf::from,
    );
    let mut zips: HashMap<String, ZipDoAcervo> = HashMap::new();
    let mut registros = Vec::new();
    let mut total_cards = 0;
    let mut formularios_confrontados = 0;
    for instalacao in INSTALACOES {
        let mut cards = std::fs::read_dir(raiz.join(instalacao))
            .expect("pasta das cassettes")
            .map(|item| item.expect("card").path())
            .filter(|p| {
                p.file_name()
                    .is_some_and(|s| s.to_string_lossy().starts_with("processo-atribuir-"))
                    && p.extension().is_some_and(|s| s == "md")
            })
            .collect::<Vec<_>>();
        cards.sort();
        for card in cards {
            total_cards += 1;
            let nome = archive_filename(&card).expect("origem do ZIP");
            let zip = zips.entry(nome.clone()).or_insert_with(|| {
                ZipDoAcervo::ler(&cache.join(&nome)).unwrap_or_else(|e| panic!("{nome}: {e}"))
            });
            let conferida = conferir_cassette_ate(&card, zip, 120_000_000)
                .expect("hash, schema, completude e host da cassette");
            assert_eq!(conferida.instalacao, instalacao);
            registros.extend(replay(instalacao, &conferida.json));
            formularios_confrontados += confrontar_formularios(instalacao, &conferida.json);
        }
    }
    assert_eq!(
        total_cards,
        INSTALACOES.len(),
        "uma cassette por instalação"
    );
    assert!(
        !registros.is_empty(),
        "nenhuma resposta de atribuição gravada"
    );
    assert!(
        formularios_confrontados > 0,
        "nenhum formulário GET original pôde ser confrontado com o anti-IES"
    );
    let mut saida = format!(
        "# Atribuição — replay offline #264\n\n{} cassettes, {} respostas classificadas, zero requisições reais. POST histórico não prova escrita.\n\n",
        total_cards,
        registros.len()
    );
    let _ = writeln!(
        saida,
        "{} formulários GET confrontados independentemente com o anti-IES por HTTP local (identidade, opções e rótulos); sem prova de POST/estado externo.\n",
        formularios_confrontados
    );
    tabela(
        &mut saida,
        registros.iter().map(|r| {
            (
                usize::from(r.leitor != "formulário de atribuição"),
                r.leitor,
                r.instalacao.as_str(),
                &r.resultado,
            )
        }),
    );
    let lidos = registros
        .iter()
        .filter(|r| r.resultado == Resultado::Lido)
        .count();
    let _ = writeln!(saida, "Formulários aceitos pelo leitor real: {lidos}.\n");
    publicar(&saida);
}
