// BDD-60-05: históricos GO/UFRJ/PEN1 confrontados com anti-IES.
// Corpo capturado só entra na versão e identidade efetivamente observadas.

/// Encontra o número SEI pelo mesmo `id_documento` em uma árvore gravada,
/// sem emprestar identidade ou tipo de uma fixture `UnB`.
fn documento_na_arvore(
    captura: &serde_json::Value,
    id_documento: &str,
) -> (String, String, String, String) {
    let mut identidades = Vec::new();
    for resposta in gravacao::respostas(captura) {
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
        for doc in &arvore.documentos {
            if doc.id == id_documento
                && let Some(numero) = &doc.numero_sei
            {
                identidades.push((
                    processo.clone(),
                    nup.clone(),
                    numero.clone(),
                    doc.rotulo.clone(),
                ));
            }
        }
    }
    let primeiro = identidades
        .first()
        .expect("uma árvore original deve associar o histórico ao documento")
        .clone();
    assert!(
        identidades.iter().all(|identidade| identidade == &primeiro),
        "a identidade da árvore precisa ser estável na mesma cassette"
    );
    primeiro
}

/// Só observa o corpo quando o GET da mesma identidade existe na captura.
fn corpo_da_cassette(
    captura: &serde_json::Value,
    id: &str,
    numero_observado: &str,
    exigir_corpo: bool,
) -> Option<(u32, String, String)> {
    let originais: Vec<_> = gravacao::respostas(captura)
        .into_iter()
        .filter(|r| {
            r.operacao == "documento_visualizar"
                && r.metodo == "GET"
                && r.status == 200
                && gravacao::parametro(r.item, "id_documento").contains(&id)
        })
        .collect();
    assert!(originais.len() <= 1, "um único GET de corpo por identidade");
    let conteudo = originais.first().map(|r| {
        let html = html_da_resposta(r.item).unwrap();
        let identidade = crate::documento_web::identidade_do_titulo(&html).unwrap();
        assert_eq!(identidade.numero, numero_observado);
        let versao = crate::rodape_do_documento::versao_do_rodape(&html)
            .expect("versão no rodapé original");
        let texto = crate::documento_web::texto_do_corpo(&html).unwrap();
        (versao.numero(), identidade.tipo, texto)
    });
    assert!(
        !exigir_corpo || conteudo.is_some(),
        "PEN1 precisa associar seu GET original de corpo ao id do histórico"
    );
    conteudo
}

/// Compara o único corpo original observado com a projeção HTTP do SQLite.
fn confrontar_corpo_observado_http(
    endereco: SocketAddr,
    id: &str,
    processo: &str,
    numero: &NumeroDocumento,
    conteudo: Option<&(u32, String, String)>,
) {
    if let Some((versao, tipo_original, texto_original)) = conteudo {
        let alvo = format!(
            "/sei/controlador.php?acao=documento_visualizar&id_documento={id}&id_procedimento={processo}&versao={versao}"
        );
        let resposta = pedir(endereco, "GET", &alvo);
        let (cabecalho, html) = resposta.split_once("\r\n\r\n").unwrap();
        assert!(cabecalho.starts_with("HTTP/1.1 200 OK"), "{cabecalho}");
        let identidade = crate::documento_web::identidade_do_titulo(html).unwrap();
        assert_eq!(identidade.numero.as_str(), numero.as_str());
        assert_eq!(&identidade.tipo, tipo_original);
        assert_eq!(
            crate::rodape_do_documento::versao_do_rodape(html)
                .map(crate::documento_versoes::VersaoDocumento::numero),
            Some(*versao)
        );
        let corpo = html
            .split_once("<main id='corpoEditavel'>")
            .expect("editor projetado")
            .1
            .split_once("</main>")
            .unwrap()
            .0;
        let texto = crate::documento_web::texto_do_corpo(&format!("<body>{corpo}</body>")).unwrap();
        assert_eq!(
            &texto, texto_original,
            "corpo capturado deve sobreviver ao transporte"
        );
    }
}

fn confrontar_cassette_historico_por_http(caminho_card: &str, exigir_corpo: bool) {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let card = raiz.join(caminho_card);
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/ies"),
        std::path::PathBuf::from,
    );
    let nome_zip = crate::cassette::archive_filename(&card).unwrap();
    let zip = ZipDoAcervo::ler(&cache.join(nome_zip)).unwrap();
    let captura = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
    let instalacao = gravacao::instalacao(&captura.json).unwrap();
    let respostas: Vec<_> = gravacao::respostas(&captura.json)
        .into_iter()
        .filter(|r| r.operacao == "documento_versao_listar" && r.metodo == "GET" && r.status == 200)
        .collect();
    assert_eq!(respostas.len(), 1, "usar somente a lista realmente gravada");
    let ids = gravacao::parametro(respostas[0].item, "id_documento");
    let id = *ids.first().expect("id_documento no GET original");
    assert!(
        ids.iter().all(|observado| *observado == id),
        "o histórico original não pode misturar IDs"
    );
    let (processo, nup, numero_observado, rotulo_observado) =
        documento_na_arvore(&captura.json, id);
    let numero = NumeroDocumento::novo(&numero_observado).unwrap();
    let html_original = html_da_resposta(respostas[0].item).unwrap();
    let historico_original = interpretar_historico(&html_original, &numero, id, None)
        .expect("leitor real do IES na lista original");
    let conteudo = corpo_da_cassette(&captura.json, id, &numero_observado, exigir_corpo);
    let mut revisoes = revisoes_observadas(&historico_original, "");
    if let Some((versao, _, texto)) = &conteudo {
        let revisao = revisoes
            .iter_mut()
            .find(|r| r.versao.get() == *versao)
            .expect("versão do corpo capturado existe no histórico");
        revisao.html_corpo = format!(
            "<p>{}</p>",
            texto.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        );
    }
    assert!(
        revisoes.iter().all(|r| {
            conteudo.as_ref().is_some_and(|(v, _, _)| *v == r.versao.get())
                || r.html_corpo == "<p></p>"
        }),
        "versões sem corpo capturado ficam apenas com estrutura vazia"
    );
    let quantidade = revisoes.len();
    let documento = DocumentoSimulado {
        instalacao: instalacao.clone(),
        id_documento: id.to_owned(),
        id_procedimento: processo.clone(),
        numero_processo: nup,
        numero: numero_observado,
        // GO/UFRJ não capturaram tipo no corpo; PEN1 usa a identidade original.
        tipo: conteudo
            .as_ref()
            .map_or(rotulo_observado, |(_, tipo, _)| tipo.clone()),
        revisoes,
    };
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let pedidos = 3 + usize::from(conteudo.is_some());
    let tarefa = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        guardar(&mut banco, "cassette-historico-sei", &documento).unwrap();
        for _ in 0..pedidos {
            let (mut fluxo, _) = escuta.accept().unwrap();
            anti_ies::atender_http(
                &mut fluxo,
                Some((&banco, &instalacao, "cassette-historico-sei")),
            )
            .unwrap();
        }
        assert_eq!(
            banco.query_row("SELECT count(*) FROM documento_revisao", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            i64::try_from(quantidade).unwrap(),
            "GET, recusa de identidade e POST não alteram as revisões"
        );
    });
    // A mesma lista original é lida pelo IES; a projeção HTTP do anti-IES
    // deve conservar todas as versões, usuários, unidades e datas.
    confrontar_historico_http(endereco, id, &processo, &numero, &historico_original);
    confrontar_corpo_observado_http(endereco, id, &processo, &numero, conteudo.as_ref());
    let falso_processo = format!(
        "/sei/controlador.php?acao=documento_versao_listar&id_documento={id}&id_procedimento={processo}0"
    );
    assert!(
        pedir(endereco, "GET", &falso_processo).starts_with("HTTP/1.1 404"),
        "processo divergente não pode receber o histórico do documento"
    );
    let alvo = format!(
        "/sei/controlador.php?acao=documento_versao_listar&id_documento={id}&id_procedimento={processo}"
    );
    assert!(pedir(endereco, "POST", &alvo).starts_with("HTTP/1.1 405"));
    tarefa.join().unwrap();
}

#[test]
#[ignore = "ZIP original GO SHA-verificado no IES_ACERVO_CACHE; somente offline"]
fn cassette_go_historico_preserva_metadados_e_identidade_por_http() {
    confrontar_cassette_historico_por_http(
        "cassettes/go/liberacoes-e-versoes-leitura-go--3ca10f1df295.md",
        false,
    );
}

#[test]
#[ignore = "ZIP original UFRJ SHA-verificado no IES_ACERVO_CACHE; somente offline"]
fn cassette_ufrj_historico_preserva_metadados_e_identidade_por_http() {
    confrontar_cassette_historico_por_http(
        "cassettes/ufrj/liberacoes-e-versoes-leitura-ufrj--37ca449e6b97.md",
        false,
    );
}

#[test]
#[ignore = "ZIP original PEN1 SHA-verificado no IES_ACERVO_CACHE; somente offline"]
fn cassette_pen1_historico_e_conteudo_preservam_identidade_por_http() {
    confrontar_cassette_historico_por_http(
        "cassettes/pen1/liberacoes-e-versoes-leitura-pen1--f46eada4effb.md",
        true,
    );
}
