// BDD-60-05: a tela publicada após pesquisa deve selecionar o mesmo
// documento e processo que a árvore original. Não compara tokens de sessão.

/// Lê somente o `id_documento` publicado no src do iframe, sem executar JS.
fn id_selecionado_na_tela(html: &str) -> Option<String> {
    let servido = HtmlServido::de_teste(html);
    let dom = crate::tela_html::interpretar(&servido);
    let iframes = crate::html_atributos::nos_com_id(&dom, "ifrArvore").ok()?;
    let [iframe] = iframes.as_slice() else {
        return None;
    };
    let tag = iframe.get(dom.parser())?.as_tag()?;
    let src = crate::html_atributos::valor_do_atributo(tag, "src")
        .ok()??;
    let src = crate::formulario_web::decodificar_atributo(&src);
    let consulta = src.split_once('?')?.1;
    let ids: Vec<_> = consulta
        .split('&')
        .filter_map(|par| par.split_once('='))
        .filter(|(nome, _)| *nome == "id_documento")
        .map(|(_, valor)| valor)
        .collect();
    let [id] = ids.as_slice() else {
        return None;
    };
    (!id.is_empty()).then(|| (*id).to_owned())
}

fn conferir_recusas_da_tela(endereco: SocketAddr, id: &str, processo: &str, url: &str) {
    let processo_estranho = format!(
        "/sei/controlador.php?acao=procedimento_trabalhar&id_documento={id}&id_procedimento={processo}0"
    );
    assert!(
        pedir(endereco, "GET", &processo_estranho).starts_with("HTTP/1.1 404"),
        "nao reaproveitar conteudo documental de outro processo"
    );
    assert!(
        pedir(endereco, "POST", url).starts_with("HTTP/1.1 405"),
        "o transporte de leitura nao pode aceitar POST"
    );
}

// A mesma comparação vale para as três instalações com a pesquisa por número SEI
// capturada. A identidade de cada uma vem do seu ZIP original.
fn confrontar_tela_de_pesquisa_com_cassette(caminho_card: &str) {
    use crate::arvore_processo::{
        ler_arvore, ler_tela_do_documento, nup_declarado_na_arvore,
    };

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
    let respostas = gravacao::respostas(&captura.json);

    // A cassette contém duas telas de trabalho. Só uma seleção documental
    // explícita serve para esta comparação; não forçar o caso de processo.
    let selecionadas: Vec<_> = respostas
        .iter()
        .filter(|r| {
            r.operacao == "procedimento_trabalhar" && r.metodo == "GET" && r.status == 200
        })
        .filter_map(|r| {
            let original = html_da_resposta(r.interacao).ok()?;
            let id = id_selecionado_na_tela(&original)?;
            let (tela, processo) =
                ler_tela_do_documento(&instalacao, &HtmlServido::de_teste(&original), &id, None).ok()?;
            Some((id, processo, tela))
        })
        .collect();
    assert!(
        !selecionadas.is_empty(),
        "a cassette precisa publicar pelo menos uma seleção documental no iframe"
    );
    let (id, processo, tela_original) = &selecionadas[0];

    // Identidade e número vêm da árvore gravada, não de um documento inventado.
    let (nup, numero, rotulo) = respostas
        .iter()
        .filter(|r| {
            r.operacao == "procedimento_visualizar"
                && r.metodo == "GET"
                && r.status == 200
                && gravacao::id_procedimento(r.interacao).as_deref() == Some(processo.as_str())
        })
        .find_map(|r| {
            let original = html_da_resposta(r.interacao).ok()?;
            let nup = nup_declarado_na_arvore(&original)?;
            let arvore = ler_arvore(&original, &nup, processo).ok()?;
            let documento = arvore.documentos.iter().find(|d| &d.id == id)?;
            Some((nup, documento.numero_sei.clone()?, documento.rotulo.clone()))
        })
        .expect("a arvore original precisa ligar o documento selecionado ao NUP");

    // A versão, autor, data e corpo não são confrontados nesta cassette.
    // Sentinelas explícitas satisfazem a estrutura sem fingir observação.
    let documento = DocumentoSimulado {
        instalacao: instalacao.clone(),
        id_documento: id.clone(),
        id_procedimento: processo.clone(),
        numero_processo: nup.clone(),
        numero: numero.clone(),
        tipo: rotulo,
        revisoes: vec![Revisao {
            versao: NonZeroU32::new(1).unwrap(),
            usuario: "nao_observado".to_owned(),
            unidade: tela_original.unidade.clone(),
            modificacao: "nao_observada".to_owned(),
            html_corpo: "<p></p>".to_owned(),
        }],
    };
    let (endereco, tarefa) = servir_conteudo_observado(documento, instalacao.clone(), 4);

    let url = format!("/sei/controlador.php?acao=procedimento_trabalhar&id_documento={id}");
    let resposta = pedir(endereco, "GET", &url);
    let (cabecalho, html_anti) = resposta.split_once("\r\n\r\n").unwrap();
    assert!(cabecalho.starts_with("HTTP/1.1 200 OK"));
    let (tela_anti, processo_anti) =
        ler_tela_do_documento(&instalacao, &HtmlServido::de_teste(html_anti), id, None).unwrap();
    assert_eq!(&processo_anti, processo, "iframe aponta para o mesmo processo");
    assert_eq!(
        tela_anti.unidade, tela_original.unidade,
        "a unidade vem da tela original"
    );
    // Endereços assinados completos não são comparáveis entre sessões.
    assert!(tela_anti.arvore.as_str().contains(&format!("id_documento={id}")));

    let arvore_url = format!(
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento={processo}&id_documento={id}"
    );
    let resposta = pedir(endereco, "GET", &arvore_url);
    let (cabecalho, html_anti) = resposta.split_once("\r\n\r\n").unwrap();
    assert!(cabecalho.starts_with("HTTP/1.1 200 OK"));
    let arvore_anti = ler_arvore(html_anti, &nup, processo).unwrap();
    assert_eq!(arvore_anti.processo.nup, nup);
    assert!(
        arvore_anti
            .documentos
            .iter()
            .any(|doc| &doc.id == id && doc.numero_sei.as_deref() == Some(numero.as_str())),
        "a selecao deve conservar id do documento e numero SEI da captura"
    );

    conferir_recusas_da_tela(endereco, id, processo, &url);
    tarefa.join().unwrap();
}

#[test]
#[ignore = "ZIP original UnB SHA-verificado no IES_ACERVO_CACHE; confronto offline"]
fn cassette_unb_tela_do_documento_preserva_selecao_ate_arvore_http() {
    confrontar_tela_de_pesquisa_com_cassette(
        "cassettes/unb/documento-numero-e-busca-unb--f1b386d8e7bc.md",
    );
}

#[test]
#[ignore = "ZIP original GO SHA-verificado no IES_ACERVO_CACHE; confronto offline"]
fn cassette_go_tela_do_documento_preserva_selecao_ate_arvore_http() {
    confrontar_tela_de_pesquisa_com_cassette(
        "cassettes/go/documento-numero-e-busca-go--f26b4140acb8.md",
    );
}

#[test]
#[ignore = "ZIP original UFRJ SHA-verificado no IES_ACERVO_CACHE; confronto offline"]
fn cassette_ufrj_tela_do_documento_preserva_selecao_ate_arvore_http() {
    confrontar_tela_de_pesquisa_com_cassette(
        "cassettes/ufrj/documento-numero-e-busca-ufrj--d0b23f01b824.md",
    );
}

#[test]
#[ignore = "ZIP original PEN1 SHA-verificado no IES_ACERVO_CACHE; confronto offline"]
fn cassette_pen1_tela_do_documento_preserva_selecao_ate_arvore_http() {
    confrontar_tela_de_pesquisa_com_cassette(
        "cassettes/pen1/documento-numero-e-busca-pen1--f274a10a3ee0.md",
    );
}

#[test]
#[ignore = "ZIP original PEN2 SHA-verificado no IES_ACERVO_CACHE; confronto offline"]
fn cassette_pen2_tela_do_documento_preserva_selecao_ate_arvore_http() {
    confrontar_tela_de_pesquisa_com_cassette(
        "cassettes/pen2/documento-numero-e-busca-pen2--fac3b99dd330.md",
    );
}

#[test]
#[ignore = "ZIP original PEN3 SHA-verificado no IES_ACERVO_CACHE; confronto offline"]
fn cassette_pen3_tela_do_documento_preserva_selecao_ate_arvore_http() {
    confrontar_tela_de_pesquisa_com_cassette(
        "cassettes/pen3/documento-numero-e-busca-pen3--2964627c9d6b.md",
    );
}

#[test]
#[ignore = "ZIP original PEN4 SHA-verificado no IES_ACERVO_CACHE; confronto offline"]
fn cassette_pen4_tela_do_documento_preserva_selecao_ate_arvore_http() {
    confrontar_tela_de_pesquisa_com_cassette(
        "cassettes/pen4/documento-numero-e-busca-pen4--831054375369.md",
    );
}
