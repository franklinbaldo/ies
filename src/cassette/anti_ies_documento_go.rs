// BDD-60-05: confrontos independentes de históricos GO/UFRJ com anti-IES.
// Essas gravações não têm GET de corpo: nenhuma revisão recebe texto suposto.

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

fn confrontar_cassette_historico_por_http(caminho_card: &str) {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let card = raiz.join(caminho_card);
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
        .expect("leitor real do IES na lista original GO");
    // Nem GO nem UFRJ capturaram documento_visualizar. Todos os corpos são
    // placeholders estruturais exigidos pelo banco, não observações.
    let revisoes = revisoes_observadas(&historico_original, "");
    assert!(
        revisoes.iter().all(|r| r.html_corpo == "<p></p>"),
        "não inventar conteúdo histórico não observado"
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
        guardar(&mut banco, "cassette-historico-sei", &documento).unwrap();
        for _ in 0..3 {
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
    );
}

#[test]
#[ignore = "ZIP original UFRJ SHA-verificado no IES_ACERVO_CACHE; somente offline"]
fn cassette_ufrj_historico_preserva_metadados_e_identidade_por_http() {
    confrontar_cassette_historico_por_http(
        "cassettes/ufrj/liberacoes-e-versoes-leitura-ufrj--37ca449e6b97.md",
    );
}
