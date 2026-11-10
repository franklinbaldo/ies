// BDD-60-05: confronto independente do histórico GO com o anti-IES.
// A gravação GO não contém GET de corpo: nenhuma revisão recebe texto suposto.

/// Encontra o número SEI pelo mesmo id_documento em uma árvore gravada,
/// sem emprestar identidade ou tipo de uma fixture UnB.
fn documento_go_na_arvore(
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
            if doc.id == id_documento {
                if let Some(numero) = &doc.numero_sei {
                    identidades.push((
                        processo.clone(),
                        nup.clone(),
                        numero.clone(),
                        doc.rotulo.clone(),
                    ));
                }
            }
        }
    }
    let primeiro = identidades
        .first()
        .expect("uma árvore original GO deve associar o histórico ao documento")
        .clone();
    assert!(
        identidades.iter().all(|identidade| identidade == &primeiro),
        "a identidade da árvore precisa ser estável na mesma cassette"
    );
    primeiro
}

#[test]
#[ignore = "ZIP original GO verificado no IES_ACERVO_CACHE; confronto offline no CI"]
fn cassette_go_historico_preserva_metadados_e_identidade_por_http() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let card = raiz.join("cassettes/go/liberacoes-e-versoes-leitura-go--3ca10f1df295.md");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/ies"),
        std::path::PathBuf::from,
    );
    let zip = ZipDoAcervo::ler(&cache.join("lote-9497057d512230ff.zip")).unwrap();
    let captura = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
    let instalacao = gravacao::instalacao(&captura.json).unwrap();
    let respostas: Vec<_> = gravacao::respostas(&captura.json)
        .into_iter()
        .filter(|r| r.operacao == "documento_versao_listar" && r.metodo == "GET" && r.status == 200)
        .collect();
    assert_eq!(respostas.len(), 1, "usar somente a lista realmente gravada em GO");
    let ids = gravacao::parametro(respostas[0].item, "id_documento");
    let id = *ids.first().expect("id_documento no GET GO");
    assert!(
        ids.iter().all(|observado| *observado == id),
        "o histórico original não pode misturar IDs"
    );
    let (processo, nup, numero_observado, rotulo_observado) =
        documento_go_na_arvore(&captura.json, id);
    let numero = NumeroDocumento::novo(&numero_observado).unwrap();
    let html_original = html_da_resposta(respostas[0].item).unwrap();
    let historico_original = interpretar_historico(&html_original, &numero, id, None)
        .expect("leitor real do IES na lista original GO");
    // GO não capturou documento_visualizar. Todos os corpos abaixo são
    // placeholders estruturais exigidos pelo banco, não observações.
    let revisoes = revisoes_observadas(&historico_original, "");
    assert!(
        revisoes.iter().all(|r| r.html_corpo == "<p></p>"),
        "não inventar conteúdo histórico GO"
    );
    let quantidade = revisoes.len();
    let documento = DocumentoSimulado {
        instalacao: instalacao.clone(),
        id_documento: id.to_owned(),
        id_procedimento: processo.clone(),
        numero_processo: nup,
        numero: numero_observado,
        // Só serve para preencher campo obrigatório, não é confrontado como
        // tipo de conteúdo: a cassette não traz GET de corpo.
        tipo: rotulo_observado,
        revisoes,
    };
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let tarefa = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        guardar(&mut banco, "cassette-go-historico", &documento).unwrap();
        for _ in 0..3 {
            let (mut fluxo, _) = escuta.accept().unwrap();
            anti_ies::atender_http(
                &mut fluxo,
                Some((&banco, &instalacao, "cassette-go-historico")),
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
