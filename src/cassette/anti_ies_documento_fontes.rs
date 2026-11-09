// Proveniência dos GETs e metadados do mesmo documento na cassette UnB.
// Extração incluída no módulo pai para manter os filtros de replay do CI.

/// Cruza a identidade do GET de conteúdo com uma árvore registrada na
/// mesma cassette. Sem a árvore, não se atribui um NUP ao documento.
fn processo_observado_da_arvore(
    json: &serde_json::Value,
    id_documento: &str,
    numero: &str,
) -> (String, String) {
    let mut vistos = Vec::new();
    for resposta in gravacao::respostas(json) {
        if resposta.operacao != "procedimento_visualizar"
            || resposta.metodo != "GET"
            || resposta.status != 200
        {
            continue;
        }
        let Some(id) = gravacao::id_procedimento(resposta.item) else {
            continue;
        };
        let Ok(html) = html_da_resposta(resposta.item) else {
            continue;
        };
        let Some(nup) = crate::arvore_processo::nup_declarado_na_arvore(&html) else {
            continue;
        };
        let Ok(arvore) = crate::arvore_processo::ler_arvore(&html, &nup, &id) else {
            continue;
        };
        if arvore
            .documentos
            .iter()
            .any(|doc| doc.id == id_documento && doc.numero_sei.as_deref() == Some(numero))
        {
            vistos.push((id, nup));
        }
    }
    let primeiro = vistos
        .first()
        .expect("árvore original liga DOC ao processo")
        .clone();
    assert!(vistos.iter().all(|identidade| identidade == &primeiro));
    primeiro
}

/// Reconstrói apenas versões/metadados observados, sem inventar corpo histórico.
fn revisoes_da_cassette(
    historico: &crate::documento_versoes::HistoricoDocumento,
) -> (Vec<Revisao>, Vec<u32>) {
    let atual = historico.atual().expect("histórico não vazio").numero();
    let mut revisoes = Vec::new();
    let mut versoes = Vec::new();
    for n in 1..=atual {
        let versao = crate::documento_versoes::VersaoDocumento::nova(n).unwrap();
        if let Ok(entrada) = historico.selecionar(versao) {
            versoes.push(n);
            revisoes.push(Revisao {
                versao: NonZeroU32::new(n).unwrap(),
                usuario: entrada.usuario.clone(),
                unidade: entrada.unidade.clone(),
                modificacao: entrada.modificacao.clone(),
                html_corpo: "<p>Conteúdo não observado nesta versão.</p>".to_owned(),
            });
        }
    }
    assert_eq!(revisoes.len(), historico.len());
    (revisoes, versoes)
}

/// O número e o tipo da mesma identidade vêm do conteúdo gravado, não da fixture.
fn identidade_do_conteudo_observado(json: &serde_json::Value, id: &str) -> (String, String) {
    let conteudos: Vec<_> = gravacao::respostas(json)
        .into_iter()
        .filter(|r| r.operacao == "documento_visualizar" && r.metodo == "GET" && r.status == 200)
        .collect();
    assert_eq!(conteudos.len(), 1, "conteúdo original do mesmo documento");
    let ids_conteudo = gravacao::parametro(conteudos[0].item, "id_documento");
    assert!(
        !ids_conteudo.is_empty() && ids_conteudo.iter().all(|valor| *valor == id),
        "histórico e conteúdo precisam referir o mesmo id_documento"
    );
    let identidade =
        crate::documento_web::identidade_do_titulo(&html_da_resposta(conteudos[0].item).unwrap())
            .unwrap();
    (identidade.numero, identidade.tipo)
}


/// A única versão cujo corpo foi capturado recebe metadados do histórico
/// original. Nenhuma linha da lista autoriza inventar corpos anteriores.
fn revisao_do_conteudo_observado(
    json: &serde_json::Value,
    id: &str,
    numero: &str,
    html_conteudo: &str,
    texto: &str,
) -> Revisao {
    let historicos: Vec<_> = gravacao::respostas(json)
        .into_iter()
        .filter(|r| r.operacao == "documento_versao_listar" && r.metodo == "GET" && r.status == 200)
        .collect();
    assert_eq!(historicos.len(), 1, "histórico da mesma cassette UnB");
    let ids = gravacao::parametro(historicos[0].item, "id_documento");
    assert!(
        !ids.is_empty() && ids.iter().all(|valor| *valor == id),
        "histórico e corpo devem pertencer ao mesmo documento"
    );
    let numero = crate::documento::NumeroDocumento::novo(numero).unwrap();
    let html_historico = html_da_resposta(historicos[0].item).unwrap();
    let lista = crate::documento_versoes::interpretar_historico(
        &html_historico,
        &numero,
        id,
        None,
    )
    .unwrap();
    let versao = crate::rodape_do_documento::versao_do_rodape(html_conteudo)
        .expect("rodapé de versão do GET original");
    let metadados = lista
        .selecionar(versao)
        .expect("mesma versão do corpo na lista original");
    Revisao {
        versao: NonZeroU32::new(versao.numero()).unwrap(),
        usuario: metadados.usuario.clone(),
        unidade: metadados.unidade.clone(),
        modificacao: metadados.modificacao.clone(),
        html_corpo: format!("<p>{}</p>", escapar(texto)),
    }
}
