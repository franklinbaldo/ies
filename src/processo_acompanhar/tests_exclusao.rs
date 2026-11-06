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
fn exclusao_recusa_funcao_em_script_nao_executavel() {
    // Em script de dados, acaoExcluir existe apenas como texto e o onclick não a invoca.
    // Caso sintético derivado da semântica de <script type="application/json">.
    let html = lista_observada("831").replacen("<script>", "<script type=\"application/json\">", 1);
    assert_eq!(
        conferir_oferta_exclusao(&instalacao(), &html, "10", "831"),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn exclusao_recusa_script_module_sem_funcao_global() {
    // Função declarada no escopo de module não é global para o onclick inline.
    let html = lista_observada("831").replacen("<script>", "<script type=\"module\">", 1);
    assert_eq!(
        conferir_oferta_exclusao(&instalacao(), &html, "10", "831"),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn exclusao_recusa_script_com_src_que_ignora_corpo_inline() {
    // O navegador ignora o código inline quando o script clássico possui src.
    let html =
        lista_observada("831").replacen("<script>", "<script src=\"/arquivo-exemplo.js\">", 1);
    assert_eq!(
        conferir_oferta_exclusao(&instalacao(), &html, "10", "831"),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn exclusao_aceita_script_classico_com_tipo_explicito() {
    let html = lista_observada("831").replacen("<script>", "<script type=\"text/javascript\">", 1);
    assert!(conferir_oferta_exclusao(&instalacao(), &html, "10", "831").is_ok());
}

#[test]
fn exclusao_recusa_funcao_inteira_dentro_de_comentario_de_bloco() {
    // O navegador não publica funções que aparecem apenas em /* ... */.
    let html = lista_observada("831")
        .replacen("<script>", "<script>/*", 1)
        .replacen("</script>", "*/</script>", 1);
    assert_eq!(
        conferir_oferta_exclusao(&instalacao(), &html, "10", "831"),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn exclusao_recusa_assinatura_comentada_por_linha() {
    // Um comentário não define a função global invocada pelo onclick.
    let html = lista_observada("831").replacen(
        "function acaoExcluir(id,desc){",
        "// function acaoExcluir(id,desc){",
        1,
    );
    assert_eq!(
        conferir_oferta_exclusao(&instalacao(), &html, "10", "831"),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn exclusao_recusa_funcao_declarada_apenas_em_bloco_inativo() {
    // Uma declaração dentro de if(false) não prova ação global disponível.
    let html = lista_observada("831")
        .replacen(
            "function acaoExcluir(id,desc){",
            "if(false){ function acaoExcluir(id,desc){",
            1,
        )
        .replacen("</script>", "}</script>", 1);
    assert_eq!(
        conferir_oferta_exclusao(&instalacao(), &html, "10", "831"),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn exclusao_recusa_expressao_de_funcao_nao_global() {
    // Uma função nomeada atribuída a variável não vira window.acaoExcluir.
    let html = lista_observada("831")
        .replacen(
            "function acaoExcluir(id,desc){",
            "const interna = function acaoExcluir(id,desc){",
            1,
        )
        .replacen("</script>", ";</script>", 1);
    assert_eq!(
        conferir_oferta_exclusao(&instalacao(), &html, "10", "831"),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn exclusao_aceita_comentario_antes_da_declaracao_global_real() {
    // Comentário anterior não torna inerte uma declaração global subsequente.
    let html = lista_observada("831").replacen("<script>", "<script>/* comentário sintético */", 1);
    assert!(conferir_oferta_exclusao(&instalacao(), &html, "10", "831").is_ok());
}

#[test]
fn exclusao_recusa_comandos_em_outra_funcao_apos_acao_excluir_vazia() {
    // O código fora do corpo de acaoExcluir não prova uma ação oferecida.
    // RED: a antiga busca por substrings no script inteiro aceitava isso.
    let html = lista_observada("831").replacen(
        "function acaoExcluir(id,desc){",
        "function acaoExcluir(id,desc){} function outra(id,desc){",
        1,
    );
    assert_eq!(
        conferir_oferta_exclusao(&instalacao(), &html, "10", "831"),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn exclusao_recusa_trechos_inalcancaveis_fora_da_funcao() {
    let html = lista_observada("831").replacen(
        "function acaoExcluir(id,desc){",
        "function acaoExcluir(id,desc){} if(false){",
        1,
    );
    assert_eq!(
        conferir_oferta_exclusao(&instalacao(), &html, "10", "831"),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn exclusao_recusa_codigo_morto_ao_redor_da_confirmacao() {
    // O formulário não deve ser tido como oferecido se o ramo observado
    // está protegido por uma condição adicional não comprovada.
    let html = lista_observada("831")
        .replacen(
            "function acaoExcluir(id,desc){",
            "function acaoExcluir(id,desc){if(false){",
            1,
        )
        .replacen("</script>", "}</script>", 1);
    assert_eq!(
        conferir_oferta_exclusao(&instalacao(), &html, "10", "831"),
        Err(ErroAcompanhar::ExclusaoNaoOferecida)
    );
}

#[test]
fn exclusao_aceita_chave_de_bloco_dentro_do_texto_da_confirmacao() {
    // Delimitadores em strings não encerram o corpo da função.
    let html = lista_observada("831").replace("Excluir?", "Excluir } ?");
    assert!(conferir_oferta_exclusao(&instalacao(), &html, "10", "831").is_ok());
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

#[test]
fn previa_deixar_com_funcao_inteiramente_comentada_recusa_sem_post() {
    // Caso adversarial da mesma tela UnB: oferta textual inerte não
    // autoriza escrita e não provoca nenhuma requisição além dos 4 GETs.
    use crate::processo_acompanhar::fluxo::{ErroPrevia, Pedido, previa};
    use crate::processo_concluir::tests_apoio::{
        UNIDADE, abertura, absoluta, ambiente, arvore, get, ok, pedidos_da_abertura, rodar_com,
    };
    use crate::propriedades_processo::tests_apoio::acao_da_barra;

    let gerenciar = format!(
        "controlador.php?acao=acompanhamento_gerenciar&id_procedimento=10&infra_hash={}",
        "a".repeat(64)
    );
    let arvore_html = arvore(&[UNIDADE], &[acao_da_barra(&gerenciar)]);
    let mut respostas = abertura(arvore_html);
    let html = lista_observada("831")
        .replacen("<script>", "<script>/*", 1)
        .replacen("</script>", "*/</script>", 1);
    respostas.push(ok(html));
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
    assert!(matches!(
        resultado,
        Err(ErroPrevia::Estrutura(ErroAcompanhar::ExclusaoNaoOferecida))
    ));
    let mut esperados = pedidos_da_abertura();
    esperados.push(get(&absoluta(&gerenciar)));
    assert_eq!(pedidos, esperados);
    assert_eq!(pedidos.len(), 4);
    assert_eq!(
        pedidos
            .iter()
            .filter(|pedido| pedido.metodo == "POST")
            .count(),
        0
    );
}
