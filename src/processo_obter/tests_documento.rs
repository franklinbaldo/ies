//! RED/GREEN de `ies documento DOC` pela Web (Refs #60): sequência exata
//! das requisições, aprendizado no índice e uma recusa tipada por caso
//! observado (conteúdo, sem acesso, cancelado, duplicidade,
//! número com zeros diferente, título de outro documento). Telas
//! SINTÉTICAS com a forma das cassettes `documento-numero-e-busca-*`.
use super::tests_apoio::*;
use super::*;
use crate::arvore_processo::tests::{NUP, arvore, no_documento, no_processo};
use crate::arvore_processo::tests_conteudo::{
    HASH_CONTEUDO, com_src, src_da_arvore_do_documento, src_visualizar,
};
use crate::arvore_processo::tests_tela::tela_do_processo;
use crate::documento::NumeroDocumento;
use crate::erro_html::ErroHtml;
use crate::transporte::sessao::CookieDaSessao;

const DOC: &str = "0000101";
const ID_DOC: &str = "21";
const CORPO_DOC: &str = "txtPesquisaRapida=0000101";

fn tela_do_documento() -> String {
    tela_do_processo(UNIDADE, &[src_da_arvore_do_documento(ID, ID_DOC)])
        .replace("</body>", &format!("{}</body>", formulario_rapido()))
}

fn arvore_com(src: &str) -> String {
    com_src(
        &arvore(&[no_processo(0, ID, NUP), no_documento(1, ID_DOC, ID, DOC)]),
        1,
        src,
    )
}

fn conteudo(numero: &str) -> String {
    format!(
        "<html><head><title>SEI/EXEMPLO - {numero} - Despacho</title></head>\
         <body><p>Texto do despacho.</p></body></html>"
    )
}

fn url_conteudo() -> String {
    absoluta(&src_visualizar(ID_DOC))
}

fn rodar_doc(
    amb: &Ambiente,
    numero: &str,
    respostas: Vec<Resposta>,
) -> (Result<DocumentoObtido, ErroDocumentoObter>, Vec<Pedido>) {
    let url = url_controle();
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &CookieDaSessao::ler(&amb.cookie),
    };
    let numero = NumeroDocumento::novo(numero).unwrap();
    let pedido = PedidoDoDocumento {
        numero: &numero,
        unidade: None,
        anexo: PedidoDoAnexo::default(),
        processo: None,
    };
    let mut roteiro = Roteiro::novo(respostas);
    let r = obter_documento(&sessao, &pedido, &amb.indice, &mut roteiro);
    assert!(
        roteiro.respostas_restantes() == 0,
        "respostas não consumidas"
    );
    (r, roteiro.pedidos)
}

/// Controle, POST, tela e árvore: as quatro primeiras respostas.
fn ate_a_arvore(arvore: String) -> Vec<Resposta> {
    vec![
        ok(controle(&[("7", "OUTRO-1")])),
        r302(&trabalhar_rapida_documento(ID_DOC)),
        ok(tela_do_documento()),
        ok(arvore),
    ]
}

#[test]
fn conteudo_pelo_controle_em_cinco_requisicoes_e_aprende_o_caminho() {
    let amb = ambiente();
    let mut respostas = ate_a_arvore(arvore_com(&src_visualizar(ID_DOC)));
    respostas.push(ok(conteudo(DOC)));
    let (r, pedidos) = rodar_doc(&amb, DOC, respostas);
    let d = r.unwrap();
    assert_eq!(
        pedidos,
        [
            get(&url_controle()),
            post(&absoluta(&action_rapida()), CORPO_DOC),
            get(&absoluta(&trabalhar_rapida_documento(ID_DOC))),
            get(&absoluta(&src_da_arvore_do_documento(ID, ID_DOC))),
            get(&url_conteudo()),
        ]
    );
    assert_eq!(d.numero, DOC);
    assert_eq!(d.id_documento, ID_DOC);
    assert_eq!(d.rotulo, "Despacho 0000101");
    assert_eq!(d.tipo, "Despacho");
    assert_eq!(d.processo.nup, NUP);
    assert_eq!(d.processo.id, ID);
    assert_eq!(d.unidade.as_deref(), Some(UNIDADE));
    assert_eq!(d.texto, "Texto do despacho.");
    assert_eq!(d.requisicoes, 5);
    assert_eq!(d.natureza.icone.as_deref(), Some("documento_interno"));
    let debug = format!("{d:?}");
    for segredo in [HASH_CONTEUDO.to_owned(), hash('b'), hash('c')] {
        assert!(!debug.contains(&segredo), "{debug}");
    }
    let (inst, g) = (instalacao(), geracao());
    assert_eq!(amb.indice.id_por_nup(&inst, NUP).as_deref(), Some(ID));
    let link = |acao, objeto| {
        amb.indice
            .link(&inst, chave(&g, acao, objeto))
            .map(|u| u.url().to_owned())
    };
    assert_eq!(
        link("procedimento_visualizar", ID),
        Some(absoluta(&src_da_arvore_do_documento(ID, ID_DOC)))
    );
    assert_eq!(
        link("protocolo_pesquisa_rapida", ""),
        Some(absoluta(&action_rapida()))
    );
}

#[test]
fn segundo_documento_usa_a_pesquisa_guardada_em_quatro() {
    let amb = ambiente();
    let mut primeira = ate_a_arvore(arvore_com(&src_visualizar(ID_DOC)));
    primeira.push(ok(conteudo(DOC)));
    rodar_doc(&amb, DOC, primeira).0.unwrap();
    // Sem o número no índice (fatia 2: com ele, 2 pelo link da árvore).
    amb.indice.esquecer_documento(&instalacao(), DOC);
    let (r, pedidos) = rodar_doc(
        &amb,
        DOC,
        vec![
            r302(&trabalhar_rapida_documento(ID_DOC)),
            ok(tela_do_documento()),
            ok(arvore_com(&src_visualizar(ID_DOC))),
            ok(conteudo(DOC)),
        ],
    );
    assert_eq!(r.unwrap().requisicoes, 4);
    assert_eq!(pedidos[0], post(&absoluta(&action_rapida()), CORPO_DOC));
}

fn recusa(arvore: String) -> ErroDocumentoObter {
    let amb = ambiente();
    let (r, pedidos) = rodar_doc(&amb, DOC, ate_a_arvore(arvore));
    assert_eq!(pedidos.len(), 4, "o conteúdo não é pedido");
    r.unwrap_err()
}

#[test]
fn sem_acesso_e_cancelado_sao_recusas_tipadas_sem_get_do_conteudo() {
    assert_eq!(
        recusa(arvore_com("")),
        ErroDocumentoObter::ConteudoNaoPublicadoNaArvore { requisicoes: 4 }
    );
    let cancelado = arvore_com("")
        .replace("svg/documento_interno.svg", "svg/documento_cancelado.svg")
        .replace(
            "De: pessoa@exemplo.gov.br\\nPara: outra@exemplo.gov.br",
            "Documento Cancelado: teste",
        );
    assert_eq!(recusa(cancelado), ErroDocumentoObter::DocumentoCancelado);
}

#[test]
fn duplicidade_e_numero_diferente_na_arvore() {
    let duplicada = com_src(
        &arvore(&[
            no_processo(0, ID, NUP),
            no_documento(1, ID_DOC, ID, DOC),
            no_documento(2, "22", ID, DOC),
        ]),
        1,
        &src_visualizar(ID_DOC),
    );
    assert_eq!(
        recusa(duplicada),
        ErroDocumentoObter::Html(ErroHtml::repetido("numero_sei_repetido_na_arvore"))
    );
    // Zeros à esquerda são parte do número: 101 não é 0000101.
    let outro = com_src(
        &arvore(&[no_processo(0, ID, NUP), no_documento(1, ID_DOC, ID, "101")]),
        1,
        &src_visualizar(ID_DOC),
    );
    assert_eq!(
        recusa(outro),
        ErroDocumentoObter::NumeroSeiDaArvoreDiferenteDoPedido
    );
    let ausente = arvore(&[no_processo(0, ID, NUP), no_documento(1, "22", ID, DOC)]);
    assert_eq!(
        recusa(ausente),
        ErroDocumentoObter::Conteudo(
            crate::arvore_processo::ErroConteudoNaArvore::DocumentoAusenteDaArvore
        )
    );
}

#[test]
fn titulo_de_outro_documento_e_recusado() {
    let amb = ambiente();
    let mut respostas = ate_a_arvore(arvore_com(&src_visualizar(ID_DOC)));
    respostas.push(ok(conteudo("0000999")));
    let (r, _) = rodar_doc(&amb, DOC, respostas);
    assert_eq!(r.unwrap_err(), ErroDocumentoObter::TituloDeOutroDocumento);
}

#[test]
fn pesquisa_sem_302_e_tela_de_outro_documento() {
    let amb = ambiente();
    let (_r, pedidos) = rodar_doc(
        &amb,
        DOC,
        vec![
            ok(controle(&[])),
            ok("<html><body>pesquisa</body></html>".into()),
        ],
    );
    assert_eq!(pedidos.len(), 2);
    let amb = ambiente();
    let mut respostas = ate_a_arvore(String::new());
    respostas[2] = ok(tela_do_processo(
        UNIDADE,
        &[src_da_arvore_do_documento(ID, "22")],
    ));
    respostas.pop();
    let (_r, _) = rodar_doc(&amb, DOC, respostas);
}

#[test]
fn propriedades_consulta_apenas_o_cadastro_servido_sem_escrita() {
    // Dado documento interno com barra documento_alterar e cadastro compatível
    // com Todos tests/paginas_reais/documento_alterar.html @323eb01,
    // Quando peço --com-propriedades, então faço um único GET adicional,
    // confiro os IDs do formulário e não envio POST de cadastro.
    use crate::arvore_processo::tests_barra_documento::{HASH_VERSOES, barra, com_barra};
    use crate::propriedades_documento::{Nivel, PropriedadesDocumento};

    let amb = ambiente();
    let arvore = arvore_com(&src_visualizar(ID_DOC));
    let arvore = com_barra(&arvore, 1, &barra(ID, ID_DOC, None));
    let mut respostas = ate_a_arvore(arvore);
    respostas.push(ok(conteudo(DOC)));
    respostas.push(ok(r#"<form id="frmDocumentoCadastro" method="post">
           <input id="hdnIdDocumento" value="21">
           <input id="hdnIdProcedimento" value="10">
           <label id="lblSerieTitulo">Despacho</label>
           <input id="txtDescricao" value="Cadastro">
           <input id="txtNomeArvore" value="Despacho de teste">
           <textarea id="txaObservacoes">observações</textarea>
           <input name="rdoNivelAcesso" value="0" checked>
           </form>"#
        .to_owned()));
    let url = url_controle();
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &CookieDaSessao::ler(&amb.cookie),
    };
    let numero = NumeroDocumento::novo(DOC).unwrap();
    let pedido = PedidoDoDocumento {
        numero: &numero,
        unidade: None,
        anexo: PedidoDoAnexo::default(),
        processo: None,
    };
    let mut roteiro = Roteiro::novo(respostas);
    let r = obter_documento_com_propriedades(
        &sessao,
        &pedido,
        PedidoDeVersoes::default(),
        &amb.indice,
        &mut roteiro,
        true,
    );
    assert_eq!(roteiro.respostas_restantes(), 0);
    let lido = r.unwrap();
    assert_eq!(lido.requisicoes, 6);
    assert_eq!(roteiro.pedidos.len(), 6);
    let link = format!(
        "controlador.php?acao=documento_alterar&id_procedimento={ID}&id_documento={ID_DOC}&infra_hash={HASH_VERSOES}"
    );
    assert_eq!(roteiro.pedidos[5], get(&absoluta(&link)));
    match lido.propriedades {
        PropriedadesDocumento::Lidas(p) => {
            assert_eq!(p.descricao, "Cadastro");
            assert_eq!(p.tipo, "Despacho");
            assert_eq!(p.nivel_de_acesso, Some(Nivel::Publico));
        }
        outra => panic!("cadastro publicado deveria ser lido: {outra:?}"),
    }
}

#[test]
fn propriedades_do_documento_fechado_consultam_formulario_publicado() {
    // Dado UFRJ SEI 4.1: processo fechado oferece documento_consultar.
    // Quando executo ies documento DOC --com-propriedades,
    // Então leio o cadastro por um GET e identifico a fonte da consulta.
    use crate::arvore_processo::tests_barra_documento::{HASH_VERSOES, barra, com_barra};
    use crate::propriedades_documento::PropriedadesDocumento;

    let amb = ambiente();
    let barra =
        barra(ID, ID_DOC, None).replace("acao=documento_alterar", "acao=documento_consultar");
    let arvore = com_barra(&arvore_com(&src_visualizar(ID_DOC)), 1, &barra);
    let mut respostas = ate_a_arvore(arvore);
    respostas.push(ok(conteudo(DOC)));
    respostas.push(ok(r#"<form id="frmDocumentoCadastro">
            <input id="hdnIdDocumento" value="21">
            <input id="hdnIdProcedimento" value="10">
            <label id="lblSerieTitulo">Ofício</label>
            <input id="txtDescricao" value="Consulta do fechado">
            <input id="txtNomeArvore" value="Ofício 0011519">
            <textarea id="txaObservacoes">Somente consulta</textarea>
            <input name="rdoNivelAcesso" value="0" checked disabled>
            </form>"#
        .to_owned()));
    let url = url_controle();
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &CookieDaSessao::ler(&amb.cookie),
    };
    let numero = NumeroDocumento::novo(DOC).unwrap();
    let pedido = PedidoDoDocumento {
        numero: &numero,
        unidade: None,
        anexo: PedidoDoAnexo::default(),
        processo: None,
    };
    let mut roteiro = Roteiro::novo(respostas);
    let lido = obter_documento_com_propriedades(
        &sessao,
        &pedido,
        PedidoDeVersoes::default(),
        &amb.indice,
        &mut roteiro,
        true,
    )
    .unwrap();
    let link = format!(
        "controlador.php?acao=documento_consultar&id_procedimento={ID}&id_documento={ID_DOC}&infra_hash={HASH_VERSOES}"
    );
    assert_eq!(roteiro.pedidos.len(), 6);
    assert_eq!(lido.requisicoes, 6);
    assert_eq!(roteiro.pedidos[5], get(&absoluta(&link)));
    assert_eq!(roteiro.respostas_restantes(), 0);
    match lido.propriedades {
        PropriedadesDocumento::Lidas(p) => {
            assert_eq!(p.fonte, "documento_consultar");
            assert_eq!(p.descricao, "Consulta do fechado");
        }
        outra => panic!("cadastro de consulta deveria ser lido: {outra:?}"),
    }
}

#[test]
fn propriedades_sem_link_publicado_nao_consultam_cadastro() {
    // Dado um documento com conteúdo mas sem link documento_alterar na barra,
    // Quando executo ies documento DOC --com-propriedades,
    // Então os metadados ficam null e não há GET adicional ao cadastro.
    use crate::propriedades_documento::PropriedadesDocumento;

    let amb = ambiente();
    let mut respostas = ate_a_arvore(arvore_com(&src_visualizar(ID_DOC)));
    respostas.push(ok(conteudo(DOC)));
    let url = url_controle();
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &CookieDaSessao::ler(&amb.cookie),
    };
    let numero = NumeroDocumento::novo(DOC).unwrap();
    let pedido = PedidoDoDocumento {
        numero: &numero,
        unidade: None,
        anexo: PedidoDoAnexo::default(),
        processo: None,
    };
    let mut roteiro = Roteiro::novo(respostas);
    let resultado = obter_documento_com_propriedades(
        &sessao,
        &pedido,
        PedidoDeVersoes::default(),
        &amb.indice,
        &mut roteiro,
        true,
    )
    .unwrap();
    assert_eq!(resultado.propriedades, PropriedadesDocumento::NaoPublicadas);
    assert_eq!(resultado.requisicoes, 5);
    assert_eq!(roteiro.pedidos.len(), 5);
    assert_eq!(roteiro.respostas_restantes(), 0);
}
