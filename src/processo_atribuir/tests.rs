use super::{ErroAtribuir as E, ler_tela, planejar};
const ID: &str = "50693";
const HTML: &str = r#"<html><body>
<form id="frmAtividadeAtribuir" method="post"
 action="controlador.php?acao=procedimento_atribuicao_cadastrar&acao_origem=procedimento_atribuicao_cadastrar&id_procedimento=50693&infra_hash=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa">
 <input type="hidden" name="hdnInfraTipoPagina" value="2">
 <button type="submit" name="sbmSalvar" value="Salvar">Salvar</button>
 <button type="button" name="btnCancelar" value="Cancelar">Cancelar</button>
 <select id="selAtribuicao" name="selAtribuicao">
  <option value="null" selected>&nbsp;</option>
  <option value="100000039">Instrutor1 - Instrutor1</option>
  <option value="100000035">aluno2 - aluno2</option>
  <option value="100000036">aluno20 - aluno20</option>
  <option value="100000004">1055542 - Rodrigo de Freitas Nogueira</option>
 </select>
 <input type="hidden" name="hdnIdProtocolo" value="50693">
</form></body></html>"#;

// Caso de Todos tests/test_ciclo_atribuir_010.py @323eb01
#[test]
fn tela_servida_identifica_usuarios_e_valores_do_select() {
    let tela = ler_tela(HTML, ID).unwrap();
    assert_eq!(tela.usuarios.len(), 4);
    assert_eq!(tela.usuarios[1].login, "aluno2");
    assert_eq!(tela.usuarios[1].id, "100000035");
}

// Caso de Todos tests/test_ciclo_atribuir_010.py @323eb01: exato, não prefixo.
#[test]
fn login_exato_nao_se_confunde_com_prefixo_ou_nome() {
    let tela = ler_tela(HTML, ID).unwrap();
    assert_eq!(
        planejar(&tela, Some("aluno2"), Some("aluno20")),
        Ok("100000035".into())
    );
    assert_eq!(
        planejar(&tela, Some("aluno"), Some("aluno20")),
        Err(E::UsuarioNaoAtribuivel)
    );
    assert_eq!(
        planejar(&tela, Some("Rodrigo de Freitas Nogueira"), Some("aluno2")),
        Err(E::UsuarioNaoAtribuivel)
    );
}

// Caso de Todos tests/test_ciclo_atribuir_010.py @323eb01: remover envia null.
#[test]
fn remover_escolhe_item_nulo_sem_escrita_desnecessaria() {
    let tela = ler_tela(HTML, ID).unwrap();
    assert_eq!(planejar(&tela, None, Some("aluno2")), Ok("null".into()));
    assert_eq!(
        planejar(&tela, None, Some("")),
        Err(E::AtribuicaoJaAplicada)
    );
}

// Não converter falta de observação em ausência de atribuição.
#[test]
fn estado_nao_observado_falha_fechado() {
    let tela = ler_tela(HTML, ID).unwrap();
    assert_eq!(
        planejar(&tela, Some("aluno2"), None),
        Err(E::SituacaoNaoObservada)
    );
    assert_eq!(
        planejar(&tela, Some("aluno2"), Some("aluno2")),
        Err(E::AtribuicaoJaAplicada)
    );
}

// Caso de Todos tests/test_ciclo_atribuir_010.py @323eb01.
#[test]
fn protocolo_de_outro_processo_nao_e_aceito() {
    assert_eq!(
        ler_tela(
            &HTML.replace(
                "name=\"hdnIdProtocolo\" value=\"50693\"",
                "name=\"hdnIdProtocolo\" value=\"999\""
            ),
            ID
        ),
        Err(E::TelaForaDaForma)
    );
}

#[test]
fn opcoes_repetidas_geram_ambiguidade_sem_escolher_a_primeira() {
    let html = HTML.replace("<option value=\"100000035\">aluno2 - aluno2</option>", "<option value=\"100000035\">aluno2 - aluno2</option><option value=\"100000077\">aluno2 - Outro Nome</option>");
    let tela = ler_tela(&html, ID).unwrap();
    assert_eq!(
        planejar(&tela, Some("aluno2"), Some("aluno20")),
        Err(E::UsuarioAmbiguo)
    );
}

// Caso de Todos tests/test_ciclo_atribuir_010.py @323eb01: null disabled.
#[test]
fn opcao_nula_desabilitada_nao_permissa_envio() {
    let html = HTML.replace(
        "value=\"null\" selected",
        "value=\"null\" selected disabled",
    );
    assert_eq!(ler_tela(&html, ID), Err(E::TelaForaDaForma));
}

#[test]
fn formulario_get_e_select_multiple_sao_recusados() {
    assert_eq!(
        ler_tela(&HTML.replace("method=\"post\"", "method=\"get\""), ID),
        Err(E::TelaForaDaForma)
    );
    assert_eq!(
        ler_tela(
            &HTML.replace("name=\"selAtribuicao\"", "name=\"selAtribuicao\" multiple"),
            ID
        ),
        Err(E::TelaForaDaForma)
    );
}

#[test]
fn subcodigos_publicos_do_leitor_sao_estaveis() {
    assert_eq!(E::TelaForaDaForma.subcodigo(), "tela_da_atribuicao_fora_da_forma");
    assert_eq!(E::UsuarioNaoAtribuivel.subcodigo(), "usuario_nao_atribuivel");
    assert_eq!(E::UsuarioAmbiguo.subcodigo(), "usuario_ambiguo_na_atribuicao");
    assert_eq!(E::SituacaoNaoObservada.subcodigo(), "situacao_da_atribuicao_nao_observada");
    assert_eq!(E::AtribuicaoJaAplicada.subcodigo(), "atribuicao_ja_aplicada");
}
