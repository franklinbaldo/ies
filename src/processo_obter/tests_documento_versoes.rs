//! RED/GREEN de `ies documento DOC --versao N` e da foto `DOC.vN.html`
//! (Refs #88, #207; contracts/cli/documento-obter-versoes.md): sequência e
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
use crate::transporte::sessao::CookieDaSessao;

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
        r302(&trabalhar_rapida_documento(ID_DOC)),
        ok(tela_do_documento()),
        ok(arvore),
    ]
}

fn primeiros() -> Vec<Pedido> {
    vec![
        get(&url_controle()),
        post(&absoluta(&action_rapida()), "txtPesquisaRapida=0000101"),
        get(&absoluta(&trabalhar_rapida_documento(ID_DOC))),
        get(&absoluta(&src_da_arvore_do_documento(ID, ID_DOC))),
    ]
}

fn rodar(
    respostas: Vec<Resposta>,
    versoes: PedidoDeVersoes,
) -> (Result<DocumentoObtido, ErroDocumentoObter>, Vec<Pedido>) {
    rodar_em(&ambiente(), None, respostas, versoes)
}

/// Como [`rodar`], num ambiente dado (índice) e com `--processo`.
fn rodar_em(
    amb: &Ambiente,
    processo: Option<&str>,
    respostas: Vec<Resposta>,
    versoes: PedidoDeVersoes,
) -> (Result<DocumentoObtido, ErroDocumentoObter>, Vec<Pedido>) {
    let url = url_controle();
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &CookieDaSessao::ler(&amb.cookie),
    };
    let numero = NumeroDocumento::novo(DOC).unwrap();
    let nup = processo.map(|p| crate::processo::NumeroProcesso::novo(p).unwrap());
    let pedido = PedidoDoDocumento {
        numero: &numero,
        unidade: None,
        processo: nup.as_ref(),
        anexo: PedidoDoAnexo::default(),
    };
    let mut roteiro = Roteiro::novo(respostas);
    let r = obter_documento_com_versoes(&sessao, &pedido, versoes, &amb.indice, &mut roteiro);
    assert_eq!(roteiro.respostas_restantes(), 0, "respostas não consumidas");
    (r, roteiro.pedidos)
}

fn versao(n: u32) -> PedidoDeVersoes {
    PedidoDeVersoes {
        cache: false,
        versao: Some(VersaoDocumento::nova(n).unwrap()),
        foto: false,
    }
}

/// `--saida DIR` sem `--versao` (#207).
const FOTO: PedidoDeVersoes = PedidoDeVersoes {
    cache: false,
    versao: None,
    foto: true,
};

fn versao_da_foto(versoes: &VersoesDoDocumento) -> Option<u32> {
    versoes.foto.as_ref().map(|f| f.versao.numero())
}

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
    assert_eq!(d.versoes.foto, None);
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

/// #207: com `--saida`, a foto é a da versão lida, atual ou histórica,
/// assinada ou não; o custo continua G + 2.
#[test]
fn versao_n_com_saida_fotografa_a_versao_lida_mesmo_historica() {
    let mut pedido = versao(2);
    pedido.foto = true;
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(lista(&[2, 1])));
    respostas.push(ok(conteudo_v(DOC, "Atual.", 2)));
    let (r, _) = rodar(respostas, pedido);
    let foto = r.unwrap().versoes.foto.unwrap();
    assert_eq!(foto.versao.numero(), 2);
    assert!(foto.html.contains("<p>Atual.</p>"));
    let mut pedido = versao(1);
    pedido.foto = true;
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('S'), true));
    respostas.push(ok(lista(&[2, 1])));
    respostas.push(ok(conteudo_v(DOC, "Antiga.", 1)));
    let (r, pedidos) = rodar(respostas, pedido);
    assert_eq!(pedidos.len(), 6);
    let d = r.unwrap();
    assert_eq!(versao_da_foto(&d.versoes), Some(1));
    assert!(d.versoes.foto.unwrap().html.contains("<p>Antiga.</p>"));
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

/// Integração com a fatia 3 do #60: o externo é lido, mas `--versao N`
/// é recusado pela barra (sem a lista) antes do GET do anexo.
#[test]
fn externo_com_versao_e_recusado_antes_do_anexo() {
    let html = com_src(
        &arvore(&[no_processo(0, ID, NUP), no_documento(1, ID_DOC, ID, DOC)]),
        1,
        &src_anexo(),
    );
    let (r, pedidos) = rodar(ate_a_arvore(html), versao(1));
    assert_eq!(pedidos.len(), 4, "nenhum GET do anexo");
    let erro = r.unwrap_err();
    assert_eq!(erro, ErroDocumentoObter::ListaDeVersoesNaoPublicadaNaArvore);
    assert_eq!(erro.subcodigo(), "lista_de_versoes_nao_publicada_na_arvore");
}

#[test]
fn foto_pelo_rodape_em_cinco_sem_a_lista() {
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(conteudo_v(DOC, "Atual.", 3)));
    let (r, pedidos) = rodar(respostas, FOTO);
    let mut esperado = primeiros();
    esperado.push(get(&absoluta(&src_visualizar(ID_DOC))));
    assert_eq!(pedidos, esperado);
    let d = r.unwrap();
    assert_eq!(d.requisicoes, 5);
    assert_eq!(n(&d.versoes), (Some(3), Ok(3)));
    assert_eq!(versao_da_foto(&d.versoes), Some(3));
    assert!(d.versoes.foto.unwrap().html.contains("<p>Atual.</p>"));
}

#[test]
fn foto_sem_rodape_le_a_lista_em_seis_e_sem_lista_recusa() {
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(conteudo(DOC, "Atual.")));
    respostas.push(ok(lista(&[1, 2, 3])));
    let (r, pedidos) = rodar(respostas, FOTO);
    let mut esperado = primeiros();
    esperado.push(get(&absoluta(&src_visualizar(ID_DOC))));
    esperado.push(get(&absoluta(&href_versoes(ID, ID_DOC))));
    assert_eq!(pedidos, esperado);
    let d = r.unwrap();
    assert_eq!(n(&d.versoes), (None, Ok(3)));
    assert_eq!(versao_da_foto(&d.versoes), Some(3));
    // Sem rodapé e sem lista na barra: recusa tipada, nenhum GET a mais.
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), false));
    respostas.push(ok(conteudo(DOC, "Atual.")));
    let (r, pedidos) = rodar(respostas, FOTO);
    assert_eq!(pedidos.len(), 5);
    let erro = r.unwrap_err();
    assert_eq!(erro, ErroDocumentoObter::ListaDeVersoesNaoPublicadaNaArvore);
    assert_eq!(erro.subcodigo(), "lista_de_versoes_nao_publicada_na_arvore");
}

/// A foto não depende da edição declarada na barra (assinado, sem
/// edição); só o `infra_hash` no HTML servido a recusa.
#[test]
fn foto_independe_da_edicao_e_recusa_link_assinado() {
    for edicao in [Some('S'), None] {
        let mut respostas = ate_a_arvore(arvore_com_barra(edicao, true));
        respostas.push(ok(conteudo_v(DOC, "Atual.", 2)));
        let (r, pedidos) = rodar(respostas, FOTO);
        assert_eq!(pedidos.len(), 5, "{edicao:?}");
        assert_eq!(versao_da_foto(&r.unwrap().versoes), Some(2), "{edicao:?}");
    }
    let comlink = conteudo_v(
        DOC,
        "<a href=\"controlador.php?acao=x&infra_hash=1\">x</a>",
        2,
    );
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(comlink));
    let (r, pedidos) = rodar(respostas, FOTO);
    assert_eq!(pedidos.len(), 5);
    let erro = r.unwrap_err();
    assert_eq!(erro, ErroDocumentoObter::ConteudoComLinkAssinado);
    assert_eq!(erro.subcodigo(), "conteudo_com_link_assinado");
}

// BDD-88-01: a opcao local de cache nao deve tornar inacessivel uma
// versao N que o SEI ja serviu. URL assinada no corpo nunca vai ao cache.
#[test]
fn bdd_88_01_conteudo_assinado_nao_impede_leitura_com_cache_ativo() {
    let com_link = conteudo_v(
        DOC,
        "<p>Texto da versao 1.</p><a href=\"controlador.php?acao=x&amp;infra_hash=assinado\">detalhe</a>",
        1,
    );
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(lista(&[2, 1])));
    respostas.push(ok(com_link));
    let mut pedido = versao(1);
    pedido.cache = true;
    let (resultado, pedidos) = rodar(respostas, pedido);
    assert_eq!(pedidos.len(), 6);
    let documento = resultado.expect("cache opcional nao pode impedir leitura de versao N");
    assert!(documento.texto.contains("Texto da versao 1."));
    assert_eq!(n(&documento.versoes), (Some(1), Ok(2)));
    assert!(documento.versoes.cache_html.is_none());
    assert!(documento.versoes.foto.is_none());

    let com_link = conteudo_v(
        DOC,
        "<p>Texto atual.</p><a href=\"controlador.php?acao=x&amp;infra_hash=assinado\">detalhe</a>",
        2,
    );
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(com_link));
    let pedido = PedidoDeVersoes {
        cache: true,
        ..PedidoDeVersoes::default()
    };
    let (resultado, pedidos) = rodar(respostas, pedido);
    assert_eq!(pedidos.len(), 5);
    let documento = resultado.expect("cache opcional nao pode impedir leitura atual");
    assert!(documento.texto.contains("Texto atual."));
    assert_eq!(n(&documento.versoes), (Some(2), Ok(2)));
    assert!(documento.versoes.cache_html.is_none());
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
    assert_eq!(d.versoes.foto, None);
    let mut respostas = ate_a_arvore(html.clone());
    respostas.push(ok(conteudo(DOC, "Atual.")));
    let (r, _) = rodar(respostas, PedidoDeVersoes::default());
    assert_eq!(
        r.unwrap().versoes.atual,
        Err(SemVersaoAtual::NaoServidaNoConteudo)
    );
    let (r, pedidos) = rodar(ate_a_arvore(html), versao(1));
    assert_eq!(pedidos.len(), 4);
    assert_eq!(r.unwrap_err().subcodigo(), "barras_repetidas_na_arvore");
}

/// Integração com a fatia 2 do #60: a versão N parte da árvore que o
/// caminho abriu. Pelo link guardado da árvore, 1 + 2 GET; pelo
/// `--processo NUP` (Controle, tela, árvore), 3 + 2 GET.
#[test]
fn versao_n_pelo_link_guardado_e_pelo_processo_informado() {
    let amb = ambiente();
    let mut respostas = ate_a_arvore(arvore_com_barra(Some('N'), true));
    respostas.push(ok(conteudo_v(DOC, "Atual.", 2)));
    let (r, _) = rodar_em(&amb, None, respostas, PedidoDeVersoes::default());
    assert_eq!(r.unwrap().requisicoes, 5);
    let respostas = vec![
        ok(arvore_com_barra(Some('N'), true)),
        ok(lista(&[2, 1])),
        ok(conteudo_v(DOC, "Antiga.", 1)),
    ];
    let (r, pedidos) = rodar_em(&amb, None, respostas, versao(1));
    let d = r.unwrap();
    assert_eq!(
        pedidos,
        [
            get(&absoluta(&src_da_arvore_do_documento(ID, ID_DOC))),
            get(&absoluta(&href_versoes(ID, ID_DOC))),
            get(&url_da_versao(1)),
        ]
    );
    assert_eq!(
        (d.requisicoes, d.resolvido_por),
        (3, ResolvidoPor::IndiceLink)
    );
    assert_eq!(n(&d.versoes), (Some(1), Ok(2)));

    let amb = ambiente();
    let respostas = vec![
        ok(controle(&[(ID, NUP)])),
        ok(trabalhar(ID)),
        ok(arvore_com_barra(Some('N'), true)),
        ok(lista(&[2, 1])),
        ok(conteudo_v(DOC, "Antiga.", 1)),
    ];
    let (r, pedidos) = rodar_em(&amb, Some(NUP), respostas, versao(1));
    let d = r.unwrap();
    assert_eq!(pedidos.len(), 5);
    assert_eq!(pedidos[3], get(&absoluta(&href_versoes(ID, ID_DOC))));
    assert_eq!((d.requisicoes, d.resolvido_por), (5, ResolvidoPor::Caixa));
    assert_eq!(n(&d.versoes), (Some(1), Ok(2)));
}
