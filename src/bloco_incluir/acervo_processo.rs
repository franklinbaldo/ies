use super::{DesfechoDaInclusao, ErroInclusaoProcessual, desfecho_sem_escrita};
use crate::bloco::TipoBloco;
use crate::bloco_detalhe::ler_detalhe;
use crate::bloco_estado::EstadoDoBloco;
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
use crate::processo::NumeroProcesso;
use crate::tela_html::HtmlServido;
use std::path::Path;

#[test]
#[ignore = "ZIP original PEN1 SHA-verificado no IES_ACERVO_CACHE; replay offline"]
fn cassette_pen1_bloco_interno_distingue_ausencia_de_inclusao_ja_confirmada() {
    let cache = std::env::var_os("IES_ACERVO_CACHE").expect("IES_ACERVO_CACHE");
    let zip = ZipDoAcervo::ler(&Path::new(&cache).join("todos-sei-cassettes-pen1.zip")).unwrap();
    let card = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("cassettes/pen1/bloco-interno-processos-pen1--69795bb274d6.md");
    let cassette = conferir_cassette_ate(&card, &zip, 20_000_000).unwrap();
    let interacoes = cassette.json["interactions"].as_array().unwrap();
    let ler = |indice: usize| {
        let interacao = &interacoes[indice];
        assert_eq!(
            interacao["request"]["operation"],
            "rel_bloco_protocolo_listar"
        );
        assert_eq!(interacao["response"]["status_code"], 200);
        let ids = gravacao::parametro(interacao, "id_bloco");
        assert!(!ids.is_empty() && ids.iter().all(|id| *id == "68"));
        let html = HtmlServido::de_teste(&html_da_resposta(interacao).unwrap());
        ler_detalhe(&html, "68", TipoBloco::Interno, EstadoDoBloco::Gerado).unwrap()
    };
    let nup = NumeroProcesso::novo("043430/2026-63").unwrap();
    let antes = ler(11);
    assert_eq!(antes.total_declarado, 0);
    // O POST precisa pertencer ao intervalo entre os dois detalhes.
    let respostas = gravacao::respostas(&cassette.json);
    let posts: Vec<_> = respostas
        .iter()
        .filter(|r| (12..22).contains(&r.passo) && r.operacao == "rel_bloco_protocolo_cadastrar")
        .collect();
    let [post] = posts.as_slice() else {
        panic!("esperado um cadastro entre os dois detalhes");
    };
    assert_eq!(post.metodo, "POST");
    assert_eq!(post.status, 302);
    let url = gravacao::url_sem_consulta(&cassette.json, post.interacao);
    let consulta = gravacao::localizacao(post.interacao)
        .and_then(|loc| crate::formulario_web::consulta_do_controlador(loc, &url))
        .and_then(|q| crate::consulta_do_controlador::ConsultaDoControlador::ler(q).ok())
        .expect("302 para o mesmo controlador");
    assert!(consulta.acao() == Ok("arvore_visualizar"));
    assert!(consulta.unico("id_bloco") == Ok("68"));
    // A última árvore anterior ao cadastro deve ser a do NUP observado.
    let arvore = respostas
        .iter()
        .rev()
        .find(|r| (12..post.passo).contains(&r.passo) && r.operacao == "procedimento_visualizar")
        .expect("árvore anterior ao cadastro");
    assert_eq!(arvore.status, 200);
    let id = gravacao::id_procedimento(arvore.interacao).expect("id da árvore servida");
    let html = html_da_resposta(arvore.interacao).unwrap();
    crate::arvore_processo::ler_arvore(&html, nup.as_str(), &id).unwrap();
    assert!(matches!(
        crate::arvore_processo::ler_oferta_de_bloco(&html),
        Ok(true)
    ));
    assert!(matches!(
        desfecho_sem_escrita(&antes, &nup, true),
        Err(ErroInclusaoProcessual::InclusaoDeProcessoIndisponivel),
    ));
    let depois = ler(22);
    assert_eq!(depois.total_declarado, 1);
    assert!(matches!(
        desfecho_sem_escrita(&depois, &nup, true),
        Ok(DesfechoDaInclusao::JaEstava {
            confirmacao_pedida: true,
        }),
    ));
}

#[test]
#[ignore = "ZIP original PEN1 SHA-verificado no IES_ACERVO_CACHE; replay offline"]
fn cassette_pen1_bloco_de_reuniao_vincula_oferta_e_pertencimento() {
    let cache = std::env::var_os("IES_ACERVO_CACHE").expect("IES_ACERVO_CACHE");
    let zip = ZipDoAcervo::ler(&Path::new(&cache).join("todos-sei-cassettes-pen1.zip")).unwrap();
    let card = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("cassettes/pen1/bloco-reuniao-ciclo-pen1--6f7d8329b52c.md");
    let cassette = conferir_cassette_ate(&card, &zip, 20_000_000).unwrap();
    let respostas = gravacao::respostas(&cassette.json);
    let posts: Vec<_> = respostas
        .iter()
        .filter(|r| r.operacao == "rel_bloco_protocolo_cadastrar")
        .collect();
    let [post] = posts.as_slice() else {
        panic!("esperado um cadastro no ciclo de reunião");
    };
    assert_eq!(post.metodo, "POST");
    assert_eq!(post.status, 302);
    let url = gravacao::url_sem_consulta(&cassette.json, post.interacao);
    let consulta = gravacao::localizacao(post.interacao)
        .and_then(|loc| crate::formulario_web::consulta_do_controlador(loc, &url))
        .and_then(|q| crate::consulta_do_controlador::ConsultaDoControlador::ler(q).ok())
        .expect("302 para o mesmo controlador");
    assert_eq!(consulta.acao(), Ok("arvore_visualizar"));
    let bloco = consulta.unico("id_bloco").unwrap();
    let detalhe_do_bloco = |r: &&gravacao::Resposta<'_>| {
        let ids = gravacao::parametro(r.interacao, "id_bloco");
        r.operacao == "rel_bloco_protocolo_listar"
            && r.status == 200
            && !ids.is_empty()
            && ids.iter().all(|id| *id == bloco)
    };
    let ler = |r: &gravacao::Resposta<'_>| {
        let html = HtmlServido::de_teste(&html_da_resposta(r.interacao).unwrap());
        ler_detalhe(&html, bloco, TipoBloco::Reuniao, EstadoDoBloco::Gerado).unwrap()
    };
    let antes = ler(respostas
        .iter()
        .take(post.passo)
        .rev()
        .find(detalhe_do_bloco)
        .expect("detalhe anterior do mesmo bloco"));
    let depois = ler(respostas
        .iter()
        .skip(post.passo + 1)
        .take_while(|r| r.operacao != "rel_bloco_protocolo_excluir")
        .find(detalhe_do_bloco)
        .expect("detalhe posterior antes da retirada"));
    assert_eq!(antes.itens.len(), antes.total_declarado);
    assert_eq!(depois.itens.len(), depois.total_declarado);
    assert_eq!(depois.total_declarado, antes.total_declarado + 1);
    let novos: Vec<_> = depois
        .itens
        .iter()
        .filter(|item| !antes.itens.iter().any(|a| a.processo == item.processo))
        .collect();
    let [novo] = novos.as_slice() else {
        panic!("esperado exatamente um NUP novo no detalhe");
    };
    assert!(
        antes
            .itens
            .iter()
            .all(|a| { depois.itens.iter().any(|d| d.processo == a.processo) })
    );
    let nup = NumeroProcesso::novo(&novo.processo).unwrap();
    let arvore = respostas
        .iter()
        .take(post.passo)
        .rev()
        .find(|r| r.operacao == "procedimento_visualizar")
        .expect("árvore anterior ao cadastro");
    assert_eq!(arvore.status, 200);
    let id = gravacao::id_procedimento(arvore.interacao).expect("id da árvore servida");
    let html = html_da_resposta(arvore.interacao).unwrap();
    crate::arvore_processo::ler_arvore(&html, nup.as_str(), &id).unwrap();
    assert_eq!(crate::arvore_processo::ler_oferta_de_bloco(&html), Ok(true));
    assert!(matches!(
        desfecho_sem_escrita(&antes, &nup, false),
        Ok(DesfechoDaInclusao::Previa),
    ));
    assert!(matches!(
        desfecho_sem_escrita(&depois, &nup, true),
        Ok(DesfechoDaInclusao::JaEstava {
            confirmacao_pedida: true
        }),
    ));
}
