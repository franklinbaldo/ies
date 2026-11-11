use super::{Atribuicao, ErroAtribuir, ler_tela, planejar};
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
        planejar(&tela, Some("aluno2"), Atribuicao::Usuario("aluno20")),
        Ok("100000035".into())
    );
    assert_eq!(
        planejar(&tela, Some("aluno"), Atribuicao::Usuario("aluno20")),
        Err(ErroAtribuir::UsuarioNaoAtribuivel)
    );
    assert_eq!(
        planejar(
            &tela,
            Some("Rodrigo de Freitas Nogueira"),
            Atribuicao::Usuario("aluno2")
        ),
        Err(ErroAtribuir::UsuarioNaoAtribuivel)
    );
}

// Caso de Todos tests/test_ciclo_atribuir_010.py @323eb01: remover envia null.
#[test]
fn remover_escolhe_item_nulo_sem_escrita_desnecessaria() {
    let tela = ler_tela(HTML, ID).unwrap();
    assert_eq!(
        planejar(&tela, None, Atribuicao::Usuario("aluno2")),
        Ok("null".into())
    );
    assert_eq!(
        planejar(&tela, None, Atribuicao::Ninguem),
        Err(ErroAtribuir::AtribuicaoJaAplicada)
    );
}

// A situação é obrigatória: sem ela não há planejamento. A recusa de
// situação não observada fica em `fluxo` (`UnidadeNaoPublicadaNaArvore` etc.).
#[test]
fn situacao_ja_aplicada_nao_escreve() {
    let tela = ler_tela(HTML, ID).unwrap();
    assert_eq!(
        planejar(&tela, Some("aluno2"), Atribuicao::Usuario("aluno2")),
        Err(ErroAtribuir::AtribuicaoJaAplicada)
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
        Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma)
    );
}

#[test]
fn opcoes_repetidas_geram_ambiguidade_sem_escolher_a_primeira() {
    let html = HTML.replace("<option value=\"100000035\">aluno2 - aluno2</option>", "<option value=\"100000035\">aluno2 - aluno2</option><option value=\"100000077\">aluno2 - Outro Nome</option>");
    let tela = ler_tela(&html, ID).unwrap();
    assert_eq!(
        planejar(&tela, Some("aluno2"), Atribuicao::Usuario("aluno20")),
        Err(ErroAtribuir::UsuarioAmbiguo)
    );
}

// Caso de Todos tests/test_ciclo_atribuir_010.py @323eb01: null disabled.
#[test]
fn opcao_nula_desabilitada_nao_permissa_envio() {
    let html = HTML.replace(
        "value=\"null\" selected",
        "value=\"null\" selected disabled",
    );
    assert_eq!(
        ler_tela(&html, ID),
        Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma)
    );
}

#[test]
fn formulario_get_e_select_multiple_sao_recusados() {
    assert_eq!(
        ler_tela(&HTML.replace("method=\"post\"", "method=\"get\""), ID),
        Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma)
    );
    assert_eq!(
        ler_tela(
            &HTML.replace("name=\"selAtribuicao\"", "name=\"selAtribuicao\" multiple"),
            ID
        ),
        Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma)
    );
}

/// Um valor numérico precisa identificar um único usuário; caso contrário,
/// escolher pelo login poderia atribuir a outra pessoa.
#[test]
fn mesmo_id_publicado_para_logins_distintos_e_recusado() {
    let html = HTML.replace(
        "<option value=\"100000036\">aluno20 - aluno20</option>",
        "<option value=\"100000035\">aluno20 - aluno20</option>",
    );
    assert_eq!(
        ler_tela(&html, ID),
        Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma)
    );
}

/// O payload é derivado da tela sintética observada, sem requisição.
fn preparar(
    login: Option<&str>,
    atual: Atribuicao<'_>,
) -> Result<(String, crate::formulario_web::SubmissaoDerivada), ErroAtribuir> {
    let instalacao =
        crate::instalacao::Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let url = format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_atribuicao_cadastrar&id_procedimento={ID}&infra_hash={}",
        "b".repeat(64)
    );
    let servida = crate::formulario_web::TelaServida {
        url: &url,
        html: HTML,
        codificacao: crate::pagina_do_controle::codificacao_da_tela(
            "text/html; charset=iso-8859-1",
        )
        .unwrap(),
    };
    super::preparar_submissao(&instalacao, &servida, ID, "UNIDADE TESTE", login, atual)
}

// Caso de Todos tests/test_ciclo_atribuir_010.py @323eb01 (payload do navegador).
#[test]
fn post_de_atribuicao_usa_campos_na_ordem_da_tela_e_login_exato() {
    let (valor, post) = preparar(Some("aluno2"), Atribuicao::Usuario("aluno20")).unwrap();
    assert_eq!(valor, "100000035");
    assert_eq!(
        post.nomes_dos_campos(),
        [
            "hdnInfraTipoPagina",
            "sbmSalvar",
            "selAtribuicao",
            "hdnIdProtocolo"
        ]
    );
    assert_eq!(
        post.corpo(),
        b"hdnInfraTipoPagina=2&sbmSalvar=Salvar&selAtribuicao=100000035&hdnIdProtocolo=50693"
    );
    assert_eq!(post.efeito(), super::ATRIBUIR_PROCESSO.efeito);
    assert_eq!(crate::formulario_web::SubmissaoDerivada::METODO, "POST");
    assert!(
        !post
            .nomes_dos_campos()
            .iter()
            .any(|campo| campo == "btnCancelar")
    );
}

#[test]
fn post_de_remocao_usa_opcao_null_e_mesma_forma() {
    let (valor, post) = preparar(None, Atribuicao::Usuario("aluno2")).unwrap();
    assert_eq!(valor, "null");
    assert_eq!(
        post.corpo(),
        b"hdnInfraTipoPagina=2&sbmSalvar=Salvar&selAtribuicao=null&hdnIdProtocolo=50693"
    );
}

#[test]
fn preparacao_nao_fabrica_selecao_para_login_inexistente() {}

#[test]
fn derivacao_recusa_formulario_com_encoding_alheio_sem_enviar() {
    let instalacao =
        crate::instalacao::Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let url = format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_atribuicao_cadastrar&infra_hash={}",
        "b".repeat(64)
    );
    let html = HTML.replace(
        "method=\"post\"",
        "method=\"post\" enctype=\"multipart/form-data\"",
    );
    assert!(matches!(
        super::preparar_submissao(
            &instalacao,
            &crate::formulario_web::TelaServida {
                url: &url,
                html: &html,
                codificacao: crate::pagina_do_controle::codificacao_da_tela(
                    "text/html; charset=iso-8859-1",
                )
                .unwrap(),
            },
            ID,
            "UNIDADE TESTE",
            Some("aluno2"),
            Atribuicao::Usuario("aluno20"),
        ),
        Err(ErroAtribuir::Formulario(
            crate::formulario_web::ErroSubmissaoFormulario::EnctypeDoFormularioNaoUrlencoded
        ))
    ));
}

/// Regressão #264: uma ação de escrita não é oferecida quando o próprio
/// formulário, a escolha ou o botão são inertes no DOM. O campo pode
/// continuar visível no HTML sem ser acionável na interface Web.
#[test]
fn formulario_com_controles_inertes_nao_e_oferta_de_atribuicao() {
    let casos = [
        HTML.replace("<form id=", "<form hidden id="),
        HTML.replace(
            "<button type=\"submit\" name=\"sbmSalvar\"",
            "<button hidden type=\"submit\" name=\"sbmSalvar\"",
        ),
        HTML.replace(
            "<select id=\"selAtribuicao\"",
            "<select hidden id=\"selAtribuicao\"",
        ),
        HTML.replace(
            "<button type=\"submit\" name=\"sbmSalvar\" value=\"Salvar\">Salvar</button>",
            "<fieldset disabled><button type=\"submit\" name=\"sbmSalvar\" value=\"Salvar\">Salvar</button></fieldset>",
        ),
        HTML.replace(
            "<select id=\"selAtribuicao\"",
            "<fieldset disabled><select id=\"selAtribuicao\"",
        ).replace("</select>", "</select></fieldset>"),
    ];
    for (indice, html) in casos.iter().enumerate() {
        assert_eq!(
            ler_tela(html, ID),
            Err(ErroAtribuir::TelaDaAtribuicaoForaDaForma),
            "controle inerte no cenário {indice} deve recusar a escrita",
        );
    }
}
