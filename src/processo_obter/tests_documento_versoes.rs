//! RED/GREEN de `ies documento DOC --versao N` e da cópia de trabalho
//! (Refs #88; contracts/cli/documento-obter-versoes.md): sequência e
//! orçamento exatos das requisições, `versao_atual` só do rodapé servido
//! ou da lista lida, e uma recusa tipada por caso. Telas SINTÉTICAS com a
//! forma das cassettes `liberacoes-e-versoes-leitura-*` (barra do nó e
//! lista de versões) e do rodapé servido na PEN4 5.0.4.
use super::tests_apoio::*;
use super::*;
use crate::arvore_processo::tests::{NUP, arvore, no_documento, no_processo};
use crate::arvore_processo::tests_barra_documento::{barra, com_barra, href_versoes};
use crate::arvore_processo::tests_conteudo::{
    com_src, src_anexo, src_da_arvore_do_documento, src_visualizar,
};
use crate::arvore_processo::tests_tela::tela_do_processo;
use crate::documento::NumeroDocumento;
use crate::documento_versoes::{ErroVersao, VersaoDocumento};
use crate::rodape_do_documento::testes::rodape;

const DOC: &str = "0000101";
const ID_DOC: &str = "21";

fn tela_do_documento() -> String {
    tela_do_processo(UNIDADE, &[src_da_arvore_do_documento(ID, ID_DOC)])
        .replace("</body>", &format!("{}</body>", formulario_rapido()))
}

fn arvore_com_barra(edicao: Option<char>, com_lista: bool) -> String {
    let html = com_src(
        &arvore(&[no_processo(0, ID, NUP), no_documento(1, ID_DOC, ID, DOC)]),
        1,
        &src_visualizar(ID_DOC),
    );
    let mut texto = barra(ID, ID_DOC, edicao);
    if !com_lista {
        texto = texto.replace("documento_versao_listar", "documento_imprimir_web");
    }
    com_barra(&html, 1, &texto)
}

fn conteudo(numero: &str, texto: &str) -> String {
    format!(
        "<html><head><title>SEI/EXEMPLO - {numero} - Despacho</title></head>\
         <body><p>{texto}</p></body></html>"
    )
}

/// Conteúdo com o rodapé `Criado por …, versão N por … em ….`.
fn conteudo_v(numero: &str, texto: &str, versao: u32) -> String {
    conteudo(numero, texto).replace(
        "</body>",
        &format!("{}</body>", rodape(&versao.to_string())),
    )
}

fn lista(numeros: &[u32]) -> String {
    crate::documento_versoes::testes::pagina_servida(ID_DOC, ID_DOC, numeros)
}

/// O `href` da linha N da lista sintética, absoluto.
fn url_da_versao(n: u32) -> String {
    absoluta(&format!(
        "controlador.php?acao=documento_visualizar&acao_origem=documento_versao_listar&acao_retorno=documento_versao_listar&versao={n}&id_documento={ID_DOC}&arvore=1&id_procedimento=10&id_documento={ID_DOC}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}",
        "d".repeat(64)
    ))
}

fn ate_a_arvore(arvore: String) -> Vec<Resposta> {
    vec![
        ok(controle(&[("7", "OUTRO-1")])),
        r302(&trabalhar_rapida(ID_DOC)),
        ok(tela_do_documento()),
        ok(arvore),
    ]
}

fn primeiros() -> Vec<Pedido> {
    vec![
        get(&url_controle()),
        post(&absoluta(&action_rapida()), "txtPesquisaRapida=0000101"),
        get(&absoluta(&trabalhar_rapida(ID_DOC))),
        get(&absoluta(&src_da_arvore_do_documento(ID, ID_DOC))),
    ]
}

fn rodar(
    respostas: Vec<Resposta>,
    versoes: PedidoDeVersoes,
) -> (Result<DocumentoObtido, ErroDocumentoObter>, Vec<Pedido>) {
    let amb = ambiente();
    let url = url_controle();
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie_file: &amb.cookie,
    };
    let numero = NumeroDocumento::novo(DOC).unwrap();
    let pedido = PedidoDoDocumento {
        numero: &numero,
        unidade: None,
    };
    let mut roteiro = Roteiro::novo(respostas);
    let r = obter_documento_com_versoes(&sessao, &pedido, versoes, &amb.indice, &mut roteiro);
    assert_eq!(roteiro.respostas_restantes(), 0, "respostas não consumidas");
    (r, roteiro.pedidos)
}

fn versao(n: u32) -> PedidoDeVersoes {
    PedidoDeVersoes {
        versao: Some(VersaoDocumento::nova(n).unwrap()),
        copia: false,
    }
}

const COPIA: PedidoDeVersoes = PedidoDeVersoes {
    versao: None,
    copia: true,
};

fn n(versoes: &VersoesDoDocumento) -> (Option<u32>, Result<u32, SemVersaoAtual>) {
    (
        versoes.lida.map(VersaoDocumento::numero),
        versoes.atual.map(VersaoDocumento::numero),
    )
}

#[test]
fn versao_n_le_a_lista_e_a_linha_n_sem_o_conteudo_atual_em_seis() {
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(lista(&[2, 1])));
    respostas.push(ok(conteudo_v(DOC, "Texto da versão 1.", 1)));
    let (r, pedidos) = rodar(respostas, versao(1));
    let d = r.unwrap();
    let mut esperado = primeiros();
    esperado.push(get(&absoluta(&href_versoes(ID, ID_DOC))));
    esperado.push(get(&url_da_versao(1)));
    assert_eq!(pedidos, esperado);
    assert_eq!(d.requisicoes, 6);
    assert!(d.texto.starts_with("Texto da versão 1."));
    assert_eq!(n(&d.versoes), (Some(1), Ok(2)));
    assert_eq!(d.versoes.copia, Err(SemCopia::NaoPedida));
    assert!(!format!("{d:?}").contains(&"d".repeat(64)));
}

#[test]
fn rodape_de_outra_versao_e_recusa_e_sem_rodape_vale_a_lista() {
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(lista(&[2, 1])));
    respostas.push(ok(conteudo_v(DOC, "Atual.", 2)));
    let (r, _) = rodar(respostas, versao(1));
    let erro = r.unwrap_err();
    assert_eq!(erro, ErroDocumentoObter::VersaoServidaDiferenteDaPedida);
    assert_eq!(erro.subcodigo(), "versao_servida_diferente_da_pedida");
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(lista(&[2, 1])));
    respostas.push(ok(conteudo(DOC, "Sem rodapé.")));
    let (r, _) = rodar(respostas, versao(1));
    assert_eq!(n(&r.unwrap().versoes), (Some(1), Ok(2)));
}

#[test]
fn versao_atual_com_copia_prepara_a_copia_do_html_servido() {
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(lista(&[2, 1])));
    respostas.push(ok(conteudo_v(DOC, "Atual.", 2)));
    let mut pedido = versao(2);
    pedido.copia = true;
    let (r, _) = rodar(respostas, pedido);
    let copia = r.unwrap().versoes.copia.unwrap();
    assert_eq!(copia.versao.numero(), 2);
    assert!(copia.html.contains("<p>Atual.</p>"));
    // Versão anterior: cópia recusada com motivo, leitura mantida.
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(lista(&[2, 1])));
    respostas.push(ok(conteudo_v(DOC, "Antiga.", 1)));
    let mut pedido = versao(1);
    pedido.copia = true;
    let (r, _) = rodar(respostas, pedido);
    assert_eq!(r.unwrap().versoes.copia, Err(SemCopia::VersaoHistorica));
}

#[test]
fn versao_ausente_lista_nao_publicada_e_titulo_de_outro_documento() {
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(lista(&[2, 1])));
    let (r, pedidos) = rodar(respostas, versao(3));
    assert_eq!(pedidos.len(), 5, "a versão ausente não é pedida");
    let erro = r.unwrap_err();
    assert_eq!(
        erro,
        ErroDocumentoObter::Versoes(ErroVersao::VersaoNaoObservada)
    );
    assert_eq!(erro.subcodigo(), "versao_nao_observada");

    let (r, pedidos) = rodar(ate_a_arvore(arvore_com_barra(None, false)), versao(1));
    assert_eq!(pedidos.len(), 4, "sem link na barra, nenhum GET a mais");
    let erro = r.unwrap_err();
    assert_eq!(erro, ErroDocumentoObter::ListaDeVersoesNaoPublicadaNaArvore);
    assert_eq!(erro.subcodigo(), "lista_de_versoes_nao_publicada_na_arvore");

    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(lista(&[1])));
    respostas.push(ok(conteudo("0000999", "Outro.")));
    let (r, _) = rodar(respostas, versao(1));
    assert_eq!(r.unwrap_err(), ErroDocumentoObter::TituloDeOutroDocumento);
}

#[test]
fn externo_com_versao_e_recusado_antes_da_barra() {
    let html = com_src(
        &arvore(&[no_processo(0, ID, NUP), no_documento(1, ID_DOC, ID, DOC)]),
        1,
        &src_anexo(),
    );
    let (r, pedidos) = rodar(ate_a_arvore(html), versao(1));
    assert_eq!(pedidos.len(), 4);
    assert_eq!(r.unwrap_err(), ErroDocumentoObter::DocumentoExterno);
}

#[test]
fn copia_pelo_rodape_em_cinco_sem_a_lista() {
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(conteudo_v(DOC, "Atual.", 3)));
    let (r, pedidos) = rodar(respostas, COPIA);
    let mut esperado = primeiros();
    esperado.push(get(&absoluta(&src_visualizar(ID_DOC))));
    assert_eq!(pedidos, esperado);
    let d = r.unwrap();
    assert_eq!(d.requisicoes, 5);
    assert_eq!(n(&d.versoes), (Some(3), Ok(3)));
    assert_eq!(d.versoes.copia.unwrap().versao.numero(), 3);
}

#[test]
fn copia_sem_rodape_le_a_lista_em_seis() {
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(conteudo(DOC, "Atual.")));
    respostas.push(ok(lista(&[1, 2, 3])));
    let (r, pedidos) = rodar(respostas, COPIA);
    let mut esperado = primeiros();
    esperado.push(get(&absoluta(&src_visualizar(ID_DOC))));
    esperado.push(get(&absoluta(&href_versoes(ID, ID_DOC))));
    assert_eq!(pedidos, esperado);
    let d = r.unwrap();
    assert_eq!(n(&d.versoes), (None, Ok(3)));
    assert_eq!(d.versoes.copia.unwrap().versao.numero(), 3);
    // Sem rodapé e sem lista na barra: nenhuma cópia, nenhum GET a mais.
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), false));
    respostas.push(ok(conteudo(DOC, "Atual.")));
    let (r, pedidos) = rodar(respostas, COPIA);
    assert_eq!(pedidos.len(), 5);
    let d = r.unwrap();
    assert_eq!(
        d.versoes.atual,
        Err(SemVersaoAtual::ListaNaoPublicadaNaArvore)
    );
    assert_eq!(d.versoes.copia, Err(SemCopia::ListaNaoPublicadaNaArvore));
}

#[test]
fn copia_insegura_nao_pede_a_lista() {
    for (edicao, sem_copia) in [
        (Some('S'), SemCopia::DocumentoAssinado),
        (None, SemCopia::EdicaoNaoPublicadaNaArvore),
    ] {
        let mut respostas = ate_a_arvore(arvore_com_barra(edicao, true));
        respostas.push(ok(conteudo(DOC, "Atual.")));
        let (r, pedidos) = rodar(respostas, COPIA);
        assert_eq!(pedidos.len(), 5, "{edicao:?}");
        let d = r.unwrap();
        assert_eq!(d.versoes.atual, Err(SemVersaoAtual::NaoServidaNoConteudo));
        assert_eq!(d.versoes.copia, Err(sem_copia));
    }
    let comlink = conteudo_v(
        DOC,
        "<a href=\"controlador.php?acao=x&infra_hash=1\">x</a>",
        2,
    );
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(comlink));
    let (r, pedidos) = rodar(respostas, COPIA);
    assert_eq!(pedidos.len(), 5);
    let d = r.unwrap();
    assert_eq!(n(&d.versoes), (Some(2), Ok(2)));
    assert_eq!(d.versoes.copia, Err(SemCopia::ConteudoComLinkAssinado));
}

#[test]
fn sem_pedido_a_barra_nao_e_lida_e_versao_atual_vem_do_rodape() {
    // Barra repetida (recusa se lida) não muda a leitura sem pedido.
    let html = arvore_com_barra(Some('N'), true).replacen(
        "Nos[1].src = ",
        "Nos[1].acoes = '';\nNos[1].src = ",
        1,
    );
    let mut respostas = ate_a_arvore(html.clone());
    respostas.push(ok(conteudo_v(DOC, "Atual.", 4)));
    let (r, pedidos) = rodar(respostas, PedidoDeVersoes::default());
    assert_eq!(pedidos.len(), 5);
    let d = r.unwrap();
    assert_eq!(n(&d.versoes), (Some(4), Ok(4)));
    assert_eq!(d.versoes.copia, Err(SemCopia::NaoPedida));
    let mut respostas = ate_a_arvore(html.clone());
    respostas.push(ok(conteudo(DOC, "Atual.")));
    let (r, _) = rodar(respostas, PedidoDeVersoes::default());
    assert_eq!(
        r.unwrap().versoes.atual,
        Err(SemVersaoAtual::NaoServidaNoConteudo)
    );
    let (r, pedidos) = rodar(ate_a_arvore(html), versao(1));
    assert_eq!(pedidos.len(), 4);
    assert_eq!(
        r.unwrap_err().subcodigo(),
        "barras_do_documento_repetidas_na_arvore"
    );
}
