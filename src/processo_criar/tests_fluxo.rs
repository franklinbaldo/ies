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
        hipotese_legal: None,
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
        hipotese_legal: None,
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
        hipotese_legal: None,
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

/// Revisão da #249: a recusa HTTP do GET preserva subcódigo e status do SEI.
#[test]
fn recusa_http_do_get_preserva_status_e_nao_segue() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        Err(FalhaTransporte::StatusHttpDeErroDoServidor(
            crate::transporte::StatusHttpRecusado::sem_mensagem(503),
        )),
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
    let erro = resultado.expect_err("recusa HTTP");
    assert_eq!(erro.subcodigo(), "status_http_de_erro_do_servidor");
    assert_eq!(erro.status_http().map(|s| s.status), Some(503));
    assert_eq!(erro.saida(), 1);
    assert_eq!(requests.len(), 2);
}

/// Caso de Todos `tests/test_processo_criar_escolher_tipo_010.py` @323eb01:
/// a variante 4.1 exige um POST de seleção (só abre cadastro) e somente
/// um POST criador. Após esse segundo POST, há duas leituras de verificação.
#[test]
fn confirma_41_com_post_de_escolha_e_post_criador_unicos() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(true)),
        r302(&format!(
            "controlador.php?acao=procedimento_gerar&infra_hash={HASH}"
        )),
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
    assert_eq!(feito.requisicoes, 7);
    assert_eq!(requests.len(), 7);
    assert_eq!(
        requests.iter().filter(|req| req.metodo == "POST").count(),
        2
    );
    assert!(requests[2].url.contains("acao=procedimento_escolher_tipo"));
    assert!(requests[4].url.contains("acao=procedimento_gerar"));
    assert!(requests[5..].iter().all(|req| req.metodo == "GET"));
}

/// Segurança da confirmação: após POST, a Location não pode trazer duas
/// ações, mesmo que só uma delas coincida com a ação esperada.
/// Nenhum GET pode seguir uma Location ambígua: efeito incerto, saída 3.
#[test]
fn redirecionamento_com_duas_acoes_nao_eh_seguido() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        r302(&format!(
            "controlador.php?acao=procedimento_trabalhar&acao=procedimento_excluir&id_procedimento=10&infra_hash={HASH}"
        )),
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
    assert_eq!(resultado, Err(ErroCriar::EfeitoIncerto));
    assert_eq!(requests.len(), 4);
    assert_eq!(
        requests.iter().filter(|req| req.metodo == "POST").count(),
        1
    );
}

/// Segurança da confirmação: um `id_procedimento` inválido não é ignorado
/// quando outro identificador numérico também aparece. Uma Location com
/// dois valores do mesmo campo é ambígua e não autoriza GET.
#[test]
fn redirecionamento_com_id_invalido_duplicado_nao_eh_seguido() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        r302(&format!(
            "controlador.php?acao=procedimento_trabalhar&id_procedimento=10&id_procedimento=invalido&infra_hash={HASH}"
        )),
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
    assert_eq!(resultado, Err(ErroCriar::EfeitoIncerto));
    assert_eq!(requests.len(), 4);
    assert_eq!(
        requests.iter().filter(|req| req.metodo == "POST").count(),
        1
    );
}

/// Link servido na lista 4.0 com duas identidades não permite abrir
/// cadastro, mesmo que o primeiro ID seja numérico.
#[test]
fn lista_40_com_identidade_duplicada_recusa_sem_abrir_cadastro() {
    let amb = ambiente();
    let html = lista(false).replace(
        "id_tipo_procedimento=12&amp;infra_hash",
        "id_tipo_procedimento=12&amp;id_tipo_procedimento=invalido&amp;infra_hash",
    );
    let respostas = vec![ok(tela_controle()), ok(html)];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    assert_eq!(resultado, Err(ErroCriar::TipoSemIdentificador));
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|req| req.metodo == "GET"));
}

/// A mesma ação esperada mais uma segunda ação não pode ser aceita em link
/// de escolha; isso é recusado antes de consultar uma URL ambígua.
#[test]
fn lista_40_com_duas_acoes_recusa_sem_abrir_cadastro() {
    let amb = ambiente();
    let html = lista(false).replace(
        "acao=procedimento_gerar&amp;id_tipo_procedimento",
        "acao=procedimento_gerar&amp;acao=procedimento_excluir&amp;id_tipo_procedimento",
    );
    let respostas = vec![ok(tela_controle()), ok(html)];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    assert_eq!(resultado, Err(ErroCriar::Formulario));
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|req| req.metodo == "GET"));
}
