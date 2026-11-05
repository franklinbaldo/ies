//! Casos sintéticos da issue #245: oráculo de cenário Todos
//! `tests/test_processo_criar_escolher_tipo_010.py`, sem portar a arquitetura.
use super::*;
use crate::processo_concluir::tests_apoio::{
    UNIDADE, ambiente, arvore, controle, ok, r302, rodar_com, trabalhar,
};
use crate::transporte::FalhaTransporte;

const TIPO: &str = "Administração Geral: Ensaio";
const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn tela_controle() -> String {
    controle().replace(
        "</body>",
        &format!(
            "<a href='controlador.php?acao=procedimento_escolher_tipo&infra_hash={HASH}'>Iniciar Processo</a></body>"
        ),
    )
}

fn lista(versao_41: bool) -> String {
    let escolha = if versao_41 {
        "<a class='ancoraOpcao' href='#' onclick='escolher(12)'>Administração Geral: Ensaio</a>"
            .to_owned()
    } else {
        format!(
            "<a class='ancoraOpcao' href='controlador.php?acao=procedimento_gerar&amp;id_tipo_procedimento=12&amp;infra_hash={HASH}'>{TIPO}</a>"
        )
    };
    format!(
        "<form id='frmProcedimentoEscolherTipo' method='post' action='controlador.php?acao=procedimento_escolher_tipo&infra_hash={HASH}'>         <input type='hidden' name='hdnIdTipoProcedimento' value=''>         <input type='hidden' name='hdnFiltroTipoProcedimento' value='U'>         </form>         <a onclick=\"exibirTiposProcedimento('U')\">Todos</a>         <table id='tblTipoProcedimento'><tr><td>{escolha}</td></tr></table>"
    )
}

fn cadastro() -> String {
    format!(
        "<form id='frmProcedimentoCadastro' method='post' action='controlador.php?acao=procedimento_gerar&infra_hash={HASH}'>         <input type='hidden' name='hdnIdTipoProcedimento' value='12'>         <input type='hidden' name='hdnFlagProcedimentoCadastro' value='0'>         <input type='hidden' name='hdnAssuntos' value=''>         <input type='text' name='txtDescricao' value=''>         <input type='radio' name='rdoNivelAcesso' value='0' checked>         <input type='radio' name='rdoNivelAcesso' value='1'>         <select name='selAssuntos'><option value='21'>Assunto de Ensaio</option></select>         <select name='selHipoteseLegal'><option value='31'>Base Legal Teste</option></select>         </form>"
    )
}

fn pedido(confirmar: bool) -> PedidoCriar<'static> {
    PedidoCriar {
        tipo: TIPO,
        especificacao: "Ensaio automatizado issue 245",
        interessados: &[],
        nivel: "publico",
        unidade: None,
        confirmar,
    }
}

/// Caso de Todos `tests/test_processo_criar_escolher_tipo_010.py`: prévia 4.0
#[test]
fn previa_40_nao_tem_post_criador() {
    let amb = ambiente();
    let respostas = vec![ok(tela_controle()), ok(lista(false)), ok(cadastro())];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(false),
            rede,
        )
    });
    let realizado = resultado.unwrap();
    assert!(!realizado.confirmado);
    assert_eq!(realizado.requisicoes, 3);
    assert_eq!(requests.len(), 3);
    assert!(requests.iter().all(|req| req.metodo == "GET"));
}

/// Caso de Todos `tests/test_processo_criar_escolher_tipo_010.py`: escolha 4.1
#[test]
fn previa_41_envia_so_escolha_nao_criacao() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(true)),
        r302(&format!(
            "controlador.php?acao=procedimento_gerar&infra_hash={HASH}"
        )),
        ok(cadastro()),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(false),
            rede,
        )
    });
    let feito = resultado.unwrap();
    assert_eq!(feito.requisicoes, 4);
    assert_eq!(requests.len(), 4);
    assert_eq!(
        requests.iter().filter(|req| req.metodo == "POST").count(),
        1
    );
    assert!(requests.iter().all(|req| {
        req.metodo != "POST" || req.url.contains("acao=procedimento_escolher_tipo")
    }));
}

/// Caso de Todos `tests/test_processo_criar_escolher_tipo_010.py`: confirma 4.0
#[test]
fn confirma_40_com_uma_escrita_e_duas_leituras_de_verificacao() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        r302(&format!(
            "controlador.php?acao=procedimento_trabalhar&id_procedimento=10&infra_hash={HASH}"
        )),
        ok(trabalhar()),
        ok(arvore(&[UNIDADE], &[])),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    let feito = resultado.unwrap();
    assert!(feito.confirmado);
    assert!(feito.nup.is_some());
    assert_eq!(feito.requisicoes, 6);
    assert_eq!(requests.len(), 6);
    assert_eq!(
        requests.iter().filter(|req| req.metodo == "POST").count(),
        1
    );
}

/// Caso de Todos `tests/test_processo_criar_escolher_tipo_010.py`: falha da escrita
#[test]
fn falha_de_transporte_depois_do_post_eh_incerta_e_nao_repete() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Rede),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    assert!(matches!(resultado, Err(ErroCriar::EfeitoIncerto)));
    assert_eq!(requests.len(), 4);
    assert_eq!(
        requests.iter().filter(|req| req.metodo == "POST").count(),
        1
    );
}

/// Um interessado não resolvido deve recusar antes da escrita criadora.
#[test]
fn interessado_nao_resolvido_nao_cria() {
    let amb = ambiente();
    let interessados = vec!["Pessoa fictícia".to_owned()];
    let pedido = PedidoCriar {
        tipo: TIPO,
        especificacao: "Ensaio automatizado issue 245",
        interessados: &interessados,
        nivel: "publico",
        unidade: None,
        confirmar: true,
    };
    let respostas = vec![ok(tela_controle()), ok(lista(false)), ok(cadastro())];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido,
            rede,
        )
    });
    assert!(matches!(
        resultado,
        Err(ErroCriar::Interessado(
            crate::processo_criar_interessados::ErroInteressado::ControlesAusentes
        ))
    ));
    assert_eq!(requests.len(), 3);
    assert!(requests.iter().all(|req| req.metodo == "GET"));
}

/// Origem: Todos `tests/test_cadastro_010.py` — interessado resolvido
/// pela consulta que a tela publica, sem escrita criadora na prévia.
#[test]
fn previa_com_interessado_unico_conferido_por_ajax() {
    let amb = ambiente();
    let interessados = vec!["Pessoa de Teste".to_owned()];
    let pedido = PedidoCriar {
        tipo: TIPO,
        especificacao: "Ensaio de interessado",
        interessados: &interessados,
        nivel: "publico",
        unidade: None,
        confirmar: false,
    };
    let cadastro = cadastro().replace(
        "</form>",
        "<input type='hidden' name='hdnInteressadosProcedimento' value=''>         <input type='text' name='txtInteressadoProcedimento'>         <select name='selInteressadosProcedimento'></select>         </form>",
    ) + &format!(
        "\n<script>\nobjAutoCompletarInteressadoRI1225 = new infraAjaxAutoCompletar('hdnIdInteressadoProcedimento','txtInteressadoProcedimento','controlador_ajax.php?acao_ajax=contato_auto_completar_contexto_RI1225&infra_hash={HASH}');\n</script>"
    );
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro),
        ok(
            "<?xml version='1.0'?><itens><item id='77' descricao='Pessoa de Teste'/></itens>"
                .to_owned(),
        ),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido,
            rede,
        )
    });
    let feito = resultado.unwrap();
    assert_eq!(feito.interessados, vec!["Pessoa de Teste"]);
    assert!(!feito.confirmado);
    assert_eq!(feito.requisicoes, 4);
    assert_eq!(
        requests.iter().filter(|req| req.metodo == "POST").count(),
        1
    );
    assert!(
        requests[3]
            .url
            .contains("acao_ajax=contato_auto_completar_contexto_RI1225")
    );
}
