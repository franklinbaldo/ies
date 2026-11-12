//! #245 — cenários sintéticos derivados do cadastro do Todos @323eb01.
//! O AJAX apenas consulta hipóteses. Nunca cria processo.
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::processo_concluir::tests_apoio::{
    UNIDADE, ambiente, arvore, controle, ok, r302, rodar_com, trabalhar,
};

use crate::processo_concluir::tests_apoio::NUP;
use crate::propriedades_processo::tests_apoio::{
    acao_da_barra, link_das_propriedades, tela_restrita,
};

fn arvore_verificadora() -> String {
    arvore(
        &[UNIDADE],
        &[acao_da_barra(&link_das_propriedades(
            AcaoDoSei::ProcedimentoAlterar,
            "10",
        ))],
    )
}

fn propriedades_verificadoras() -> String {
    tela_restrita(NUP, "10")
        .replace("Tipo Sintetico: Teste", TIPO)
        .replace("Especificação sintética &amp; teste", "Ensaio restrito")
}

const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TIPO: &str = "Administracao: Ensaio";
const XML_UNICO: &str = "<?xml version='1.0'?><select><option value='null'> </option><option value='31'>Base Legal de Teste</option></select>";
const XML_DUPLO: &str = "<?xml version='1.0'?><select><option value='null'> </option><option value='31'>Base A</option><option value='32'>Base B</option></select>";

fn controle_de_criacao() -> String {
    controle().replace(
        "</body>",
        &format!("<a href='controlador.php?acao=procedimento_escolher_tipo&infra_hash={HASH}'>Iniciar Processo</a></body>"),
    )
}

fn lista() -> String {
    format!(
        "<form id='frmProcedimentoEscolherTipo' method='post' action='controlador.php?acao=procedimento_escolher_tipo&infra_hash={HASH}'>\
    <input type='hidden' name='hdnIdTipoProcedimento' value=''>\
    <input type='hidden' name='hdnFiltroTipoProcedimento' value='U'></form>\
    <a onclick=\"exibirTiposProcedimento('U')\">Todos</a>\
    <table id='tblTipoProcedimento'><tr><td>\
    <a class='ancoraOpcao' href='controlador.php?acao=procedimento_gerar&amp;id_tipo_procedimento=12&amp;infra_hash={HASH}'>{TIPO}</a>\
    </td></tr></table>"
    )
}

fn cadastro(com_ajax: bool) -> String {
    let script = if com_ajax {
        format!(
            "\n<script>\nobjAjaxHipoteseLegal = new infraAjaxMontarSelect('selHipoteseLegal','controlador_ajax.php?acao_ajax=hipotese_legal_select_nome_base_legal&infra_hash={HASH}');\n</script>"
        )
    } else {
        String::new()
    };
    format!(
        "<form id='frmProcedimentoCadastro' method='post' action='controlador.php?acao=procedimento_gerar&infra_hash={HASH}'>\
        <input type='hidden' name='hdnIdTipoProcedimento' value='12'>\
        <input type='hidden' name='hdnFlagProcedimentoCadastro' value='0'>\
        <input type='hidden' name='hdnAssuntos' value=''>\
        <input name='txtDescricao' maxlength='250'>\
        <input type='radio' name='rdoNivelAcesso' value='0'>\
        <input type='radio' name='rdoNivelAcesso' value='1'>\
        <select name='selAssuntos'><option value='21'>Assunto Teste</option></select>\
        <select name='selHipoteseLegal'><option value='null'> </option></select>\
        </form>{script}"
    )
}

fn restrito(confirmar: bool) -> PedidoCriar<'static> {
    PedidoCriar {
        tipo: TIPO,
        especificacao: "Ensaio restrito",
        interessados: &[],
        nivel: "restrito",
        unidade: None,
        hipotese_legal: None,
        confirmar,
    }
}

/// Cenário: `Todos/src/todos/documentos.py::hipoteses_do_cadastro` @323eb01.
/// No cadastro restrito o select começa vazio; uma consulta AJAX observada
/// oferece uma única opção. A prévia retorna sem POST criador.
#[test]
fn previa_restrita_liga_ajax_publicado_e_nao_cria() {
    let amb = ambiente();
    let respostas = vec![
        ok(controle_de_criacao()),
        ok(lista()),
        ok(cadastro(true)),
        ok(XML_UNICO.to_owned()),
    ];
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &restrito(false),
            rede,
        )
    });
    let feito = resultado.expect("a unica opcao AJAX deve ser observada");
    assert_eq!(feito.nivel, "restrito");
    assert!(!feito.confirmado);
    assert_eq!(feito.requisicoes, 4);
    assert_eq!(pedidos.len(), 4);
    assert_eq!(pedidos.iter().filter(|r| r.metodo == "POST").count(), 1);
    assert!(
        pedidos[3]
            .url
            .contains("acao_ajax=hipotese_legal_select_nome_base_legal")
    );
    let corpo = pedidos[3].corpo.as_deref().unwrap();
    assert!(corpo.contains("staNivelAcesso=1"));
    assert!(corpo.contains("primeiroItemValor=null"));
    assert!(corpo.contains("valorItemSelecionado="));
}

/// Cenário: `Todos/tests/test_escritas_diretas_010.py` @323eb01.
/// O AJAX não é o POST criador. Só se cria após uma hipótese unívoca
/// conferida e se verifica NUP/ID por duas leituras frescas.
#[test]
fn confirmacao_restrita_tem_ajax_uma_escrita_e_duas_releituras() {
    let amb = ambiente();
    let respostas = vec![
        ok(controle_de_criacao()),
        ok(lista()),
        ok(cadastro(true)),
        ok(XML_UNICO.to_owned()),
        r302(&format!(
            "controlador.php?acao=procedimento_trabalhar&id_procedimento=10&infra_hash={HASH}"
        )),
        ok(trabalhar()),
        ok(arvore_verificadora()),
        ok(propriedades_verificadoras()),
    ];
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &restrito(true),
            rede,
        )
    });
    let feito = resultado.expect("confirmacao verificada");
    assert!(feito.confirmado);
    assert!(feito.nup.is_some());
    assert_eq!(feito.requisicoes, 8);
    assert_eq!(pedidos.len(), 8);
    assert_eq!(pedidos.iter().filter(|r| r.metodo == "POST").count(), 2);
    assert!(
        pedidos[3]
            .url
            .contains("acao_ajax=hipotese_legal_select_nome_base_legal")
    );
    assert!(pedidos[4].url.contains("acao=procedimento_gerar"));
    assert!(
        pedidos[4]
            .corpo
            .as_deref()
            .unwrap()
            .contains("selHipoteseLegal=31")
    );
    assert!(pedidos[5..].iter().all(|r| r.metodo == "GET"));
}

/// Ausência de escolha determinística não autoriza escolher a primeira
/// hipótese, nem converter silenciosamente para nível público.
#[test]
fn ajax_com_duas_hipoteses_recusa_antes_do_post_criador() {
    let amb = ambiente();
    let respostas = vec![
        ok(controle_de_criacao()),
        ok(lista()),
        ok(cadastro(true)),
        ok(XML_DUPLO.to_owned()),
    ];
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &restrito(true),
            rede,
        )
    });
    let _erro = resultado.expect_err("duas hipoteses devem ser recusadas");
    assert_eq!(pedidos.len(), 4);
    assert_eq!(pedidos.iter().filter(|r| r.metodo == "POST").count(), 1);
    assert!(
        pedidos
            .iter()
            .all(|r| { r.metodo != "POST" || !r.url.contains("acao=procedimento_gerar") })
    );
}

/// Cenário observado no replay `UnB`: a recusa de criar com duas opções
/// deve publicar os pares exatos do AJAX, para possibilitar uma segunda
/// prévia explícita, sem qualquer POST criador.
#[test]
fn red_recusa_ajax_ambigua_mas_publica_pares_observados() {
    let amb = ambiente();
    let respostas = vec![
        ok(controle_de_criacao()),
        ok(lista()),
        ok(cadastro(true)),
        ok(XML_DUPLO.to_owned()),
    ];
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &restrito(false),
            rede,
        )
    });
    let erro = resultado.expect_err("sem ID#NOME nenhuma escrita pode ocorrer");
    assert_eq!(
        erro.opcoes_para_escolha(),
        Some(
            &[
                ("31".to_owned(), "Base A".to_owned()),
                ("32".to_owned(), "Base B".to_owned()),
            ][..],
        )
    );
    assert_eq!(pedidos.len(), 4);
    assert!(
        pedidos
            .iter()
            .all(|r| { r.metodo != "POST" || !r.url.contains("acao=procedimento_gerar") })
    );
}

/// Resposta AJAX sintética não confiável: ainda que haja opção única,
/// um identificador com formato de URL não autoriza o POST criador.
/// Orçamento de recusa: 3 GET + 1 POST AJAX consultivo, zero escrita.
#[test]
fn red_valor_ajax_com_url_recusa_sem_post_criador() {
    let amb = ambiente();
    let respostas = vec![
        ok(controle_de_criacao()),
        ok(lista()),
        ok(cadastro(true)),
        ok(
            "<select><option value='https://example.invalid/marcador'>Base A</option></select>"
                .to_owned(),
        ),
    ];
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &restrito(true),
            rede,
        )
    });
    let _erro = resultado.expect_err("identificador nao publicavel nao pode ser enviado");
    assert_eq!(pedidos.len(), 4);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "GET").count(), 3);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
    assert!(
        pedidos
            .iter()
            .all(|p| p.metodo != "POST" || !p.url.contains("acao=procedimento_gerar"))
    );
}

/// Sem declaração literal de AJAX, nenhum endpoint pode ser inventado.
#[test]
fn ajax_nao_declarado_na_tela_recusa_sem_post() {
    let amb = ambiente();
    let respostas = vec![ok(controle_de_criacao()), ok(lista()), ok(cadastro(false))];
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &restrito(true),
            rede,
        )
    });
    let _erro = resultado.expect_err("consulta nao publicada na tela");
    assert_eq!(pedidos.len(), 3);
    assert!(pedidos.iter().all(|r| r.metodo == "GET"));
}

/// Falha HTTP do AJAX ainda é anterior à escrita; conserva o status e
/// não gera uma segunda tentativa nem POST de criação.
#[test]
fn ajax_http_503_preserva_status_e_nao_cria() {
    let amb = ambiente();
    let respostas = vec![
        ok(controle_de_criacao()),
        ok(lista()),
        ok(cadastro(true)),
        Ok((503, String::new(), None)),
    ];
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &restrito(true),
            rede,
        )
    });
    let erro = resultado.expect_err("HTTP 503 deve impedir a criacao");
    assert_eq!(erro.status_http().map(|h| h.status), Some(503));
    assert_eq!(pedidos.len(), 4);
    assert_eq!(pedidos.iter().filter(|r| r.metodo == "POST").count(), 1);
    assert!(
        pedidos[3]
            .url
            .contains("acao_ajax=hipotese_legal_select_nome_base_legal")
    );
}

/// `UnB` do acervo contém múltiplas hipóteses: a identidade explícita
/// precisa ser confirmada pelo AJAX antes do único POST criador.
#[test]
fn confirmacao_restrita_escolhe_segunda_hipotese_ajax_e_verifica() {
    let amb = ambiente();
    let pedido = PedidoCriar {
        hipotese_legal: Some("32#Base B"),
        ..restrito(true)
    };
    let respostas = vec![
        ok(controle_de_criacao()),
        ok(lista()),
        ok(cadastro(true)),
        ok(XML_DUPLO.to_owned()),
        r302(&format!(
            "controlador.php?acao=procedimento_trabalhar&id_procedimento=10&infra_hash={HASH}"
        )),
        ok(trabalhar()),
        ok(arvore_verificadora()),
        ok(propriedades_verificadoras()),
    ];
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido,
            rede,
        )
    });
    let feito = resultado.expect("escolha explícita verificada");
    assert!(feito.confirmado);
    assert_eq!(feito.hipotese_legal.as_deref(), Some("32"));
    assert_eq!(feito.requisicoes, 8);
    assert_eq!(pedidos.len(), 8);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 2);
    assert!(
        pedidos[4]
            .corpo
            .as_deref()
            .unwrap()
            .contains("selHipoteseLegal=32")
    );
    assert!(pedidos[5..].iter().all(|p| p.metodo == "GET"));
}

/// Nome divergente da opção publicada: nenhum POST de escrita.
#[test]
fn nome_incorreto_da_hipotese_nao_cria_processo() {
    let amb = ambiente();
    let pedido = PedidoCriar {
        hipotese_legal: Some("32#Texto diferente"),
        ..restrito(true)
    };
    let respostas = vec![
        ok(controle_de_criacao()),
        ok(lista()),
        ok(cadastro(true)),
        ok(XML_DUPLO.to_owned()),
    ];
    let (_resultado, pedidos) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido,
            rede,
        )
    });
    assert_eq!(pedidos.len(), 4);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}
