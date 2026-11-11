//! Casos sintéticos da issue #245: oráculo de cenário Todos
//! `tests/test_processo_criar_escolher_tipo_010.py`, sem portar a arquitetura.
use super::*;
use crate::consulta_do_controlador::ErroChaveDaConsulta;
use crate::processo_concluir::tests_apoio::NUP;
use crate::processo_concluir::tests_apoio::{
    UNIDADE, ambiente, arvore, controle, ok, r302, rodar_com, trabalhar,
};
use crate::propriedades_processo::tests_apoio::{
    acao_da_barra, link_das_propriedades, tela_das_propriedades,
};
use crate::transporte::FalhaTransporte;

const TIPO: &str = "Administração Geral: Ensaio";
pub(super) const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

pub(super) fn tela_controle() -> String {
    controle().replace(
        "</body>",
        &format!(
            "<a href='controlador.php?acao=procedimento_escolher_tipo&infra_hash={HASH}'>Iniciar Processo</a></body>"
        ),
    )
}

pub(super) fn lista(versao_41: bool) -> String {
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

pub(super) fn cadastro() -> String {
    format!(
        "<form id='frmProcedimentoCadastro' method='post' action='controlador.php?acao=procedimento_gerar&infra_hash={HASH}'>         <input type='hidden' name='hdnIdTipoProcedimento' value='12'>         <input type='hidden' name='hdnFlagProcedimentoCadastro' value='0'>         <input type='hidden' name='hdnAssuntos' value=''>         <input type='text' name='txtDescricao' maxlength='250' value=''>         <input type='radio' name='rdoNivelAcesso' value='0' checked>         <input type='radio' name='rdoNivelAcesso' value='1'>         <select name='selAssuntos'><option value='21'>Assunto de Ensaio</option></select>         <select name='selHipoteseLegal'><option value='31'>Base Legal Teste</option></select>         </form>"
    )
}

pub(super) fn pedido(confirmar: bool) -> PedidoCriar<'static> {
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

fn arvore_verificadora() -> String {
    arvore(
        &[UNIDADE],
        &[acao_da_barra(&link_das_propriedades(
            "procedimento_alterar",
            "10",
        ))],
    )
}

fn propriedades_verificadoras() -> String {
    tela_das_propriedades(NUP, "10")
        .replace("Tipo Sintetico: Teste", TIPO)
        .replace(
            "Especificação sintética &amp; teste",
            "Ensaio automatizado issue 245",
        )
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
        ok(arvore_verificadora()),
        ok(propriedades_verificadoras()),
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
        ok(tela_controle()),
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
    let erro = resultado.unwrap_err();
    let diagnostico = erro.diagnostico_apos_post().unwrap();
    assert_eq!(diagnostico.candidatos_observados, Some(Vec::new()));
    assert_eq!(requests.len(), 5);
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
            crate::processo_criar_interessados::ErroInteressado::InteressadoControlesAusentes
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

/// Revisão da #249: a recusa HTTP do GET preserva o erro e o status do SEI.
#[test]
fn recusa_http_do_get_preserva_status_e_nao_segue() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        Err(FalhaTransporte::Status(
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
    assert_eq!(erro.status_http().map(|s| s.status), Some(503));
    assert_eq!(requests.len(), 2);
}

/// Caso de Todos `tests/test_processo_criar_escolher_tipo_010.py` @323eb01:
/// a versão 4.1 exige um POST de seleção (só abre cadastro) e somente
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
        ok(arvore_verificadora()),
        ok(propriedades_verificadoras()),
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
    assert_eq!(feito.requisicoes, 8);
    assert_eq!(requests.len(), 8);
    assert_eq!(
        requests.iter().filter(|req| req.metodo == "POST").count(),
        2
    );
    assert!(requests[2].url.contains("acao=procedimento_escolher_tipo"));
    assert!(requests[4].url.contains("acao=procedimento_gerar"));
    assert!(requests[5..].iter().all(|req| req.metodo == "GET"));
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
    assert!(matches!(
        resultado,
        Err(ErroCriar::Lista(ErroListaDeTipos::ChaveDoCadastroIlegivel(
            ErroChaveDaConsulta::Repetida {
                chave: "id_tipo_procedimento"
            }
        )))
    ));
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
    assert!(matches!(
        resultado,
        Err(ErroCriar::Lista(ErroListaDeTipos::ChaveDoCadastroIlegivel(
            ErroChaveDaConsulta::Repetida { chave: "acao" }
        )))
    ));
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|req| req.metodo == "GET"));
}

/// Gerados paginados (conta compartilhada do PEN1, 10/11/2026): sem
/// fotografia completa a escrita segue, e uma resposta incerta ao POST sai
/// com efeito incerto, sem reconciliar nem repetir.
#[test]
fn gerados_paginados_nao_impedem_o_post_mas_tornam_a_resposta_incerta() {
    let amb = ambiente();
    let parcial = tela_controle().replace(
        "<caption>0 registros:</caption></table>",
        "<caption>Processos (5 registros - 1 a 1):</caption>\
         <tr id='P17'><td><a class='processoVisualizado'>00000.000017/2026-17</a></td></tr></table>",
    );
    let respostas = vec![
        ok(parcial),
        ok(lista(false)),
        ok(cadastro()),
        ok("<html><body>resposta inesperada</body></html>".to_owned()),
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
    let erro = resultado.unwrap_err();
    let _diagnostico = erro.diagnostico_apos_post().expect("diagnóstico pós-POST");
    assert_eq!(requests.len(), 4);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}

/// A restrição à escrita não transforma uma prévia legítima em erro.
/// A fotografia só é exigida quando há intenção de enviar o POST criador.
#[test]
fn previa_nao_exige_fotografia_completa_de_gerados() {
    let amb = ambiente();
    let parcial = tela_controle().replace(
        "<caption>0 registros:</caption></table>",
        "<caption>Processos (5 registros - 1 a 1):</caption>\
         <tr id='P17'><td><a class='processoVisualizado'>00000.000017/2026-17</a></td></tr></table>",
    );
    let respostas = vec![ok(parcial), ok(lista(false)), ok(cadastro())];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(false),
            rede,
        )
    });
    assert!(!resultado.unwrap().confirmado);
    assert_eq!(requests.len(), 3);
    assert!(requests.iter().all(|req| req.metodo == "GET"));
}

/// #245: resposta de POST sem prova exige leitura nova do Controle, não retry.
#[test]
fn falha_do_post_dispara_uma_unica_leitura_nova_dos_gerados() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Rede),
        ok(tela_controle()),
    ];
    let (_resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    assert_eq!(requests.len(), 5);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
    assert_eq!(requests[4].metodo, "GET");
}

/// #245: falha da própria leitura nunca autoriza concluir que não criou.
#[test]
fn falha_do_post_e_da_releitura_permanece_incerta_sem_retry() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Rede),
        Err(FalhaTransporte::Status(
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
    assert_eq!(requests.len(), 5);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
    let erro = resultado.unwrap_err();
    let diagnostico = erro.diagnostico_apos_post().unwrap();
    assert_eq!(diagnostico.candidatos_observados, None);
}
