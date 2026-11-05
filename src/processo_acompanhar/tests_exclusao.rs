//! Derivado de Todos `tests/test_processo_acompanhar_010.py` @323eb01,
//! fixture da `UnB` de 15/10/2026, com valores sintéticos.
use super::*;
use crate::instalacao::Instalacao;

fn instalacao() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap()
}

fn lista_observada(id: &str) -> String {
    let gerenciar = format!(
        "controlador.php?acao=acompanhamento_gerenciar&id_procedimento=10&infra_hash={}",
        "a".repeat(64)
    );
    let excluir = format!(
        "controlador.php?acao=acompanhamento_excluir&acao_origem=acompanhamento_gerenciar&id_procedimento=10&infra_hash={}",
        "b".repeat(64)
    );
    format!(
        r##"<html><head><script>
        function acaoExcluir(id,desc){{
          if(confirm("Excluir?")){{
            document.getElementById('hdnInfraItemId').value=id;
            document.getElementById('frmGerenciarAcompanhamento').action='{excluir}';
            document.getElementById('frmGerenciarAcompanhamento').submit();
          }}
        }}
        </script></head><body>
        <form id="frmGerenciarAcompanhamento" method="post" action="{gerenciar}">
          <input type="hidden" name="hdnInfraItemId" id="hdnInfraItemId" value="">
          <table summary="Tabela de Acompanhamentos.">
            <tr><th></th><th>Grupo</th><th>Observação</th><th>Usuário</th><th>Data</th><th>Ações</th></tr>
            <tr><td><input type="checkbox" value="{id}"></td>
            <td>G</td><td>Obs.</td><td>Aluna</td><td>05/11</td>
            <td><a href="#ID-{id}" onclick="acaoExcluir('{id}','Obs.');">Excluir</a></td></tr>
          </table>
        </form></body></html>"##
    )
}

#[test]
fn exclusao_oferecida_pela_linha_e_formulario_da_mesma_tela() {
    assert!(conferir_oferta_exclusao(&instalacao(), &lista_observada("831"), "10", "831").is_ok());
}

#[test]
fn exclusao_recusa_link_que_nao_e_da_linha_escolhida() {
    let html = lista_observada("831").replace(
        "onclick=\"acaoExcluir('831','Obs.');\"",
        "onclick=\"acaoExcluir('832','Obs.');\"",
    );
    assert_eq!(
        conferir_oferta_exclusao(&instalacao(), &html, "10", "831"),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn exclusao_recusa_acao_para_outro_processo() {
    let html = lista_observada("831").replace(
        "acao=acompanhamento_excluir&acao_origem=acompanhamento_gerenciar&id_procedimento=10",
        "acao=acompanhamento_excluir&acao_origem=acompanhamento_gerenciar&id_procedimento=99",
    );
    assert!(conferir_oferta_exclusao(&instalacao(), &html, "10", "831").is_err());
}

#[test]
fn exclusao_recusa_formulario_sem_oculto_obrigatorio() {
    let html =
        lista_observada("831").replace("name=\"hdnInfraItemId\"", "name=\"campoInexistente\"");
    assert!(conferir_oferta_exclusao(&instalacao(), &html, "10", "831").is_err());
}

#[test]
fn exclusao_recusa_script_de_exclusao_ausente() {
    let html =
        lista_observada("831").replace("function acaoExcluir(id,desc)", "function outra(id,desc)");
    assert!(conferir_oferta_exclusao(&instalacao(), &html, "10", "831").is_err());
}

#[test]
fn exclusao_recusa_oculto_sem_id_associado_ao_script() {
    // Mesmo nome enviável, mas o JS escreveria em outro elemento.
    let html = lista_observada("831").replace(
        "id=\"hdnInfraItemId\" value=\"\"",
        "id=\"idDiferente\" value=\"\"",
    );
    assert_eq!(
        conferir_oferta_exclusao(&instalacao(), &html, "10", "831"),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn exclusao_recusa_id_do_oculto_duplicado_fora_do_formulario() {
    // getElementById selecionaria um elemento não pertencente ao formulário.
    let html = lista_observada("831").replace(
        "<form id=\"frmGerenciarAcompanhamento\"",
        "<input id=\"hdnInfraItemId\" type=\"hidden\" name=\"outro\" value=\"\">\n<form id=\"frmGerenciarAcompanhamento\"",
    );
    assert_eq!(
        conferir_oferta_exclusao(&instalacao(), &html, "10", "831"),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn exclusao_recusa_oculto_desabilitado() {
    // O navegador não envia um campo hidden disabled no corpo do POST.
    let html = lista_observada("831").replace(
        "id=\"hdnInfraItemId\" value=\"\"",
        "id=\"hdnInfraItemId\" value=\"\" disabled",
    );
    assert_eq!(
        conferir_oferta_exclusao(&instalacao(), &html, "10", "831"),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn previa_deixar_com_oferta_valida_somente_le_e_conta_quatro_gets() {
    // Testa o caminho completo, não somente a função pura de exclusão.
    use crate::processo_acompanhar::fluxo::{Pedido, previa};
    use crate::processo_concluir::tests_apoio::{
        UNIDADE, abertura, absoluta, ambiente, arvore, get, ok, pedidos_da_abertura, rodar_com,
    };
    use crate::propriedades_processo::tests_apoio::acao_da_barra;

    let gerenciar = format!(
        "controlador.php?acao=acompanhamento_gerenciar&id_procedimento=10&infra_hash={}",
        "a".repeat(64)
    );
    let html = arvore(&[UNIDADE], &[acao_da_barra(&gerenciar)]);
    let mut respostas = abertura(html);
    respostas.push(ok(lista_observada("831")));
    let amb = ambiente();
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, roteiro| {
        let pedido = Pedido {
            nup,
            unidade: None,
            grupo: Some("G"),
            observacao: None,
            deixar: true,
            confirmar: false,
        };
        previa(sessao, &pedido, indice, roteiro)
    });
    let plano = resultado.unwrap();
    assert_eq!(plano.operacao, "deixar");
    assert_eq!(plano.id_de_remocao.as_deref(), Some("831"));
    assert!(!plano.confirmado && !plano.alterado && !plano.ja_estava);
    assert_eq!(plano.requisicoes, 4);
    let mut esperados = pedidos_da_abertura();
    esperados.push(get(&absoluta(&gerenciar)));
    assert_eq!(pedidos, esperados);
    assert!(pedidos.iter().all(|p| p.metodo == "GET"));
}
