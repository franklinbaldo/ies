//! Orçamento exato de requisições de `processo ciente` com transporte
//! sintético; cenários do Todos 323eb01 (ciência do processo).
use super::fluxo::FalhaDoAceiteProcesso;
use super::{ErroCienciaProcesso as E, PedidoDaCiencia, ciencia_no_processo};
use crate::processo_concluir::tests_apoio::*;
use crate::transporte::FalhaTransporte;

fn botao() -> Vec<String> {
    vec![ACAO_FUNCAO.replace("concluirProcesso();", "cienciaProcesso();")]
}

fn tela() -> String {
    tela_do_no(ID)
        .replace("concluirProcesso", "cienciaProcesso")
        .replace("procedimento_concluir", "procedimento_ciencia")
}

fn link_de_ciencia() -> String {
    absoluta(&link_da_funcao(ID).replace("procedimento_concluir", "procedimento_ciencia"))
}

fn link_da_lista() -> String {
    absoluta(&link_da_funcao(ID).replace("procedimento_concluir", "protocolo_ciencia_listar"))
}

fn arvore_com_ciencias() -> String {
    let base = arvore(&[UNIDADE], &botao());
    let link_relativo = link_da_lista().replace(&format!("{ORIGEM}/"), "");
    let acao = format!(
        "NosAcoes[0] = new infraArvoreAcao(\"CIENCIAS\",\"C\",\"{ID}\",\"{link_relativo}\");"
    );
    base.replacen("</script>", &format!("{acao}\n</script>"), 1)
}

fn aceite(atividade: &str) -> String {
    format!(
        "<html><head><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento={ID}&procedimento_visualizar_ciencias=1&id_atividade={atividade}#ID-{atividade}');</script></head></html>"
    )
}

fn lista(atividade: &str) -> String {
    format!(
        "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><td><input type='checkbox' name='chkInfraItem1' value='{atividade}' /></td></tr></table></html>"
    )
}

fn rodar(
    confirmar: bool,
    respostas: Vec<Resposta>,
) -> (Result<super::CienciaDoProcesso, E>, Vec<Pedido>) {
    let amb = ambiente();
    rodar_com(&amb, respostas, |sessao, nup, indice, roteiro| {
        ciencia_no_processo(
            sessao,
            &PedidoDaCiencia {
                nup,
                unidade: None,
                confirmar,
            },
            indice,
            roteiro,
        )
    })
}

#[test]
fn previa_exatamente_tres_get_sem_get_mutador() {
    let (feito, pedidos) = rodar(false, abertura(arvore(&[UNIDADE], &botao())));
    let feito = feito.unwrap();
    assert!(!feito.confirmado);
    assert!(!feito.verificado);
    assert_eq!(feito.requisicoes, 3);
    assert_eq!(pedidos, pedidos_da_abertura());
}

#[test]
fn sem_botao_recusa_antes_de_escrever() {
    let (resultado, pedidos) = rodar(true, abertura(arvore(&[UNIDADE], vazio())));
    assert!(matches!(resultado, Err(E::CienciaNaoOferecida)));
    assert_eq!(pedidos, pedidos_da_abertura());
}

#[test]
fn confirmar_um_get_mutador_mais_arvore_e_lista_que_provam_a_atividade() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    respostas.extend([
        ok(tela()),
        ok(aceite("42457")),
        ok(arvore_com_ciencias()),
        ok(lista("42457")),
    ]);
    let (feito, pedidos) = rodar(true, respostas);
    let feito = feito.unwrap();
    assert!(feito.verificado);
    assert_eq!(feito.id_atividade.as_deref(), Some("42457"));
    assert_eq!(feito.requisicoes, 7);
    let mut esperados = pedidos_da_abertura();
    esperados.extend([
        get(&links().1),
        get(&link_de_ciencia()),
        get(&url_arvore()),
        get(&link_da_lista()),
    ]);
    assert_eq!(pedidos, esperados);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        1
    );
}

#[test]
fn falha_no_get_de_escrita_nao_dispara_retry_e_releitura_ocorre() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    respostas.extend([
        ok(tela()),
        Err(FalhaTransporte::Rede),
        ok(arvore_com_ciencias()),
    ]);
    let (resultado, pedidos) = rodar(true, respostas);
    let Err(E::NaoConfirmado { requisicoes, .. }) = resultado else {
        panic!("escrita sem aceite exige saída 3");
    };
    assert_eq!(requisicoes, 6);
    assert_eq!(pedidos.len(), 6);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        1
    );
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

#[test]
fn atividade_nao_encontrada_na_lista_e_nao_confirmado_nao_sucesso() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    respostas.extend([
        ok(tela()),
        ok(aceite("42457")),
        ok(arvore_com_ciencias()),
        ok(lista("42458")),
    ]);
    let (resultado, pedidos) = rodar(true, respostas);
    assert!(matches!(resultado, Err(E::NaoConfirmado { .. })));
    assert_eq!(pedidos.len(), 7);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        1
    );
}

#[test]
fn funcao_de_ciencia_ausente_na_tela_nao_escreve() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    respostas.push(ok(tela().replace("cienciaProcesso", "outraFuncao")));
    let (resultado, pedidos) = rodar(true, respostas);
    assert!(matches!(resultado, Err(E::Barra(_))));
    assert_eq!(pedidos.len(), 4);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        0
    );
}

/// RED #271: a Lista de Ciências preexistente não pode converter uma
/// chamada dentro de if(false) em prova do aceite desta tentativa.
#[test]
fn aceite_condicional_inativo_mantem_nao_confirmado_sem_retry() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    let html = format!(
        "<html><script>function inicializar(){{ if(false) {{ {} }} }}</script></html>",
        "atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42457#ID-42457');"
    );
    respostas.extend([ok(tela()), ok(html), ok(arvore_com_ciencias())]);
    let (resultado, pedidos) = rodar(true, respostas);
    let erro = resultado.unwrap_err();
    assert!(matches!(erro, E::NaoConfirmado { .. }));
    assert_eq!(
        pedidos.len(),
        6,
        "sem id comprovado, só releitura da árvore"
    );
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        1,
        "jamais repetir o GET mutador"
    );
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

/// RED #271: uma atividade que já estava na lista não prova esta tentativa
/// se a única chamada do aceite seria inalcançável após "return".
#[test]
fn retorno_antes_da_recarga_nao_confirma_ciencia_anterior() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    let html = format!(
        "<html><script>function inicializar(){{ return; atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento={ID}&procedimento_visualizar_ciencias=1&id_atividade=42457#ID-42457'); }}</script></html>"
    );
    // O roteiro não oferece resposta da lista: consultá-la aqui seria
    // reutilizar prova de outra tentativa e exceder as 6 requisições.
    respostas.extend([ok(tela()), ok(html), ok(arvore_com_ciencias())]);
    let (resultado, pedidos) = rodar(true, respostas);
    let erro = resultado.expect_err("atividade antiga não prova novo aceite");
    assert!(matches!(erro, E::NaoConfirmado { .. }));
    assert_eq!(
        pedidos.len(),
        6,
        "não consultar lista sem aceite comprovado"
    );
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        1,
        "GET mutador único, sem retry"
    );
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

// Regressão #271: script de dados não executa atualizarArvore. A lista
// antiga, mesmo contendo a atividade, não pode transformar o aceite
// inerte em confirmação desta única escrita.
#[test]
fn script_inerte_no_aceite_mantem_nao_confirmado_sem_consultar_lista_antiga() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    let html = aceite("42457").replacen("<script>", "<script type='application/json'>", 1);
    respostas.extend([ok(tela()), ok(html), ok(arvore_com_ciencias())]);
    let (resultado, pedidos) = rodar(true, respostas);
    let erro = resultado.expect_err("script de dados não pode publicar ciência");
    assert!(matches!(erro, E::NaoConfirmado { .. }));
    assert_eq!(pedidos.len(), 6, "não consultar lista sem atividade válida");
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        1,
        "um GET mutador, sem retry"
    );
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

/// RED #271: o segundo argumento é avaliado antes do GET conseguir
/// publicar a atividade via atualizarArvore. Não usar ciência antiga
/// da lista como prova; o GET mutador permanece único e a saída é 3.
#[test]
fn segundo_argumento_que_lanca_excecao_nao_confirma_ciencia_antiga() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    let html = aceite("42457").replacen(
        "#ID-42457');",
        "#ID-42457', (() => { throw new Error('abortar'); })());",
        1,
    );
    assert!(html.contains("throw new Error"));
    // Sem resposta para a lista: ela não pode ser consultada sem
    // comprovação de atividade executada na resposta do GET mutador.
    respostas.extend([ok(tela()), ok(html), ok(arvore_com_ciencias())]);
    let (resultado, pedidos) = rodar(true, respostas);
    let erro = resultado.expect_err("recarga não executada não comprova ciência");
    assert!(matches!(erro, E::NaoConfirmado { .. }));
    assert_eq!(pedidos.len(), 6, "não consultar lista de ciência antiga");
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        1,
        "GET potencialmente mutador único, sem retry"
    );
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

/// Caso observado no Todos `tests/test_ciclo_ciencia_010.py` @323eb01:
/// a segunda ciência do mesmo dia é recusada pela Infra. A resposta
/// HTTP 200 não autoriza reaplicar o GET nem considerar a escrita nova.
/// A mensagem fica apresentação da validação, nunca classificação.
#[test]
fn ciencia_repetida_no_dia_e_validacao_sem_retry() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    respostas.extend([
        ok(tela()),
        ok(
            "<html><body><textarea id='txaInfraValidacao'>Usuário já deu ciência no processo hoje.</textarea></body></html>"
                .to_owned(),
        ),
        ok(arvore_com_ciencias()),
    ]);
    let (resultado, pedidos) = rodar(true, respostas);
    let erro = resultado.unwrap_err();
    let E::NaoConfirmado {
        falha_do_aceite: Some(FalhaDoAceiteProcesso::ValidacaoDaInfra(recusa)),
        ..
    } = &erro
    else {
        panic!("{erro:?}");
    };
    assert_eq!(
        recusa.mensagem_do_servidor.as_deref(),
        Some("Usuário já deu ciência no processo hoje.")
    );
    assert!(matches!(
        erro,
        E::NaoConfirmado {
            falha_do_aceite: Some(FalhaDoAceiteProcesso::ValidacaoDaInfra(_)),
            requisicoes: 6,
            ..
        }
    ));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        1
    );
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

/// RED #271: a árvore publicada por um <noscript> não executado
/// não prova a ciência, ainda que a Lista antiga contenha o mesmo ID.
/// Nenhuma consulta a essa lista deve ocorrer sem aceite válido.
#[test]
fn script_inerte_em_noscript_nao_confirma_ciencia_antiga() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    let html = aceite("42457")
        .replacen("<script>", "<noscript><script>", 1)
        .replacen("</script>", "</script></noscript>", 1);
    respostas.extend([ok(tela()), ok(html), ok(arvore_com_ciencias())]);
    let (resultado, pedidos) = rodar(true, respostas);
    let erro = resultado.expect_err("script em noscript nao executa");
    assert!(matches!(erro, E::NaoConfirmado { .. }));
    assert_eq!(pedidos.len(), 6, "sem consulta a lista antiga");
    assert_eq!(
        pedidos
            .iter()
            .filter(|pedido| pedido.url == link_de_ciencia())
            .count(),
        1,
        "GET mutador unico; nunca retry"
    );
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

/// Regressão #271: a caixa do MIME não pode tornar incerto um aceite
/// clássico que publicou uma atividade; nenhuma requisição extra ocorre.
#[test]
fn mime_javascript_case_misto_mantem_prova_e_orcamento_do_processo() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    let html = aceite("42457").replacen("<script>", "<script type='TeXt/JaVaScRiPt'>", 1);
    respostas.extend([
        ok(tela()),
        ok(html),
        ok(arvore_com_ciencias()),
        ok(lista("42457")),
    ]);
    let (feito, pedidos) = rodar(true, respostas);
    let feito = feito.expect("script JavaScript com caixa mista executa normalmente");
    assert!(feito.verificado);
    assert_eq!(feito.id_atividade.as_deref(), Some("42457"));
    assert_eq!(feito.requisicoes, 7);
    assert_eq!(pedidos.len(), 7);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        1
    );
}

/// Com parâmetro MIME, o conteúdo não é script clássico em HTML.
/// Depois do único GET potencialmente mutador, falhar fechado: saída 3.
#[test]
fn mime_javascript_com_parametro_nao_libera_lista_antiga() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    let html = aceite("42457").replacen(
        "<script>",
        "<script type='text/javascript; charset=utf-8'>",
        1,
    );
    respostas.extend([ok(tela()), ok(html), ok(arvore_com_ciencias())]);
    let (resultado, pedidos) = rodar(true, respostas);
    let _erro = resultado.expect_err("tipo de script com parâmetro não prova atividade");
    assert_eq!(pedidos.len(), 6);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        1
    );
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

mod erros;
