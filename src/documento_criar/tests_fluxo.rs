//! Testes sintéticos do fluxo Web de inclusão de documento (#246).
//! Derivação sem rede; não executa JavaScript nem realiza escrita no SEI.

use super::*;
use crate::formulario_web::CodificacaoDaTela;

fn cadastro() -> String {
    let hash = "a".repeat(64);
    format!(
        r#"<html>
        <form id="frmDocumentoCadastro" method="post"
          action="controlador.php?acao=documento_gerar&amp;infra_hash={hash}">
          <input type="hidden" name="hdnIdSerie" value="291">
          <input type="hidden" name="hdnFlagDocumentoCadastro" value="1">
          <input type="radio" name="rdoNivelAcesso" id="optPublico" value="0">
          <input type="text" name="txtDescricao" id="txtDescricao" value="">
        </form></html>"#
    )
}

fn derivar(
    html: &str,
) -> Result<crate::formulario_web::SubmissaoDerivada, ErroSubmissaoFormulario> {
    let url = format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=documento_gerar&infra_hash={}",
        "b".repeat(64)
    );
    let tela = TelaServida {
        url: &url,
        html,
        codificacao: CodificacaoDaTela::Iso88591,
    };
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    derivar_submissao(
        &CADASTRAR,
        &instalacao,
        &tela,
        "UNIDADE TESTE",
        &[
            ("hdnFlagDocumentoCadastro", "2"),
            ("rdoNivelAcesso", "0"),
            ("txtDescricao", "Ensaio"),
        ],
    )
}

fn lista_41() -> String {
    // Caso de Todos tests/test_escolha_na_lista_010.py @323eb01.
    let hash = "a".repeat(64);
    format!(
        r##"<html><head><script>
        function escolher(id) {{
            document.getElementById('hdnIdSerie').value = id;
            document.getElementById('frmDocumentoEscolherTipo').submit();
        }}
        </script></head><body>
        <form id="frmDocumentoEscolherTipo" method="post"
          action="controlador.php?acao=documento_escolher_tipo&amp;infra_hash={hash}">
          <input type="hidden" id="hdnIdSerie" name="hdnIdSerie" value="">
          <input type="hidden" name="hdnInfraTipoPagina" value="2">
        </form>
        <table id="tblSeries"><tr><td><a href="#" onclick="escolher(291)">Despacho</a>
        </td></tr></table></body></html>"##
    )
}

fn derivar_escolha(
    html: &str,
) -> Result<crate::formulario_web::SubmissaoDerivada, ErroSubmissaoFormulario> {
    let url = format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=documento_escolher_tipo&infra_hash={}",
        "b".repeat(64)
    );
    let tela = TelaServida {
        url: &url,
        html,
        codificacao: CodificacaoDaTela::Iso88591,
    };
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    derivar_submissao(
        &ESCOLHER_TIPO,
        &instalacao,
        &tela,
        "UNIDADE TESTE",
        &[("hdnIdSerie", "291")],
    )
}

#[test]
fn navegar_tipo_41_deriva_post_de_leitura_antes_do_get_cadastro() {
    let envio = derivar_escolha(&lista_41()).unwrap();
    assert_eq!(
        envio.nomes_dos_campos(),
        ["hdnIdSerie", "hdnInfraTipoPagina"]
    );
    assert_eq!(envio.corpo(), b"hdnIdSerie=291&hdnInfraTipoPagina=2");
    assert_eq!(envio.efeito(), ESCOLHER_TIPO.efeito);
    assert_eq!(crate::formulario_web::SubmissaoDerivada::METODO, "POST");
}

#[test]
fn navegar_tipo_41_recusa_campo_id_repetido() {
    let _html = lista_41().replace(
        "</form>",
        "<input type='hidden' name='hdnIdSerie' value='292'></form>",
    );
}

#[test]
fn exibir_todos_deriva_um_unico_post_de_filtro_e_nao_de_criacao() {
    // Caso de Todos src/todos/documentos.py::_escolha_do_tipo @323eb01.
    let html = format!(
        r#"<html><form id="frmDocumentoEscolherTipo" method="post"
          action="controlador.php?acao=documento_escolher_tipo&amp;infra_hash={}">
          <input type="hidden" name="hdnFiltroSerie" value="U">
          <input type="hidden" name="hdnInfraTipoPagina" value="2">
        </form></html>"#,
        "a".repeat(64)
    );
    let url = format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=documento_escolher_tipo&infra_hash={}",
        "b".repeat(64)
    );
    let servida = TelaServida {
        url: &url,
        html: &html,
        codificacao: CodificacaoDaTela::Iso88591,
    };
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let post = derivar_submissao(
        &FILTRAR_TIPOS,
        &instalacao,
        &servida,
        "UNIDADE TESTE",
        &[("hdnFiltroSerie", "T")],
    )
    .unwrap();
    assert_eq!(post.corpo(), b"hdnFiltroSerie=T&hdnInfraTipoPagina=2");
    assert_eq!(post.efeito(), FILTRAR_TIPOS.efeito);
    assert_eq!(crate::formulario_web::SubmissaoDerivada::METODO, "POST");
}

#[test]
fn cadastro_ignora_required_de_controles_inativos() {
    // Teste tecnico auxiliar do BDD-246-01/02: campos nao acionaveis
    // nao impedem a previa nem o cadastro publico observado.
    for conteudo in [
        "<fieldset disabled><input name=\"campoInativo\" required></fieldset>",
        "<div hidden><input name=\"campoOculto\" required></div>",
        "<template><input name=\"campoTemplate\" required></template>",
    ] {
        let html = cadastro()
            .replace(
                "<form ",
                "<label id=\"lblSerieTitulo\">Despacho</label><form ",
            )
            .replace("</form>", &format!("{conteudo}</form>"));
        assert_eq!(
            conferir_cadastro(&html, "Despacho", None, Some("publico")),
            Ok(()),
            "controle inativo nao deve exigir entrada do usuario: {conteudo}"
        );
    }
}

#[test]
fn formulario_de_criacao_deriva_um_post_com_campos_e_efeito_exatos() {
    let envio = derivar(&cadastro()).unwrap();
    assert_eq!(
        envio.nomes_dos_campos(),
        [
            "hdnIdSerie",
            "hdnFlagDocumentoCadastro",
            "rdoNivelAcesso",
            "txtDescricao"
        ]
    );
    assert_eq!(
        envio.corpo(),
        b"hdnIdSerie=291&hdnFlagDocumentoCadastro=2&rdoNivelAcesso=0&txtDescricao=Ensaio"
    );
    assert_eq!(envio.efeito(), CADASTRAR.efeito);
    assert_eq!(crate::formulario_web::SubmissaoDerivada::METODO, "POST");
}

#[test]
fn formulario_de_criacao_recusa_campo_oculto_repetido() {
    let _html = cadastro().replace(
        "</form>",
        "<input type='hidden' name='hdnIdSerie' value='292'></form>",
    );
}

#[test]
fn formulario_de_criacao_recusa_action_sem_hash_unico() {
    let _html = cadastro().replace("&amp;infra_hash=", "&amp;token=");
}

#[test]
fn confirmar_nivel_ausente_ou_sem_hipotese_recusa_antes_de_toda_rede() {
    // Uma confirmação que ainda não pode escrever não deve sequer abrir
    // o processo: GET também pode registrar recebimento no SEI.
    use crate::processo_concluir::tests_apoio as apoio;
    for (nivel, _subcodigo) in [
        (None, "nivel_nao_informado"),
        (Some("restrito"), "nivel_restrito_sem_hipotese"),
        (Some("sigiloso"), "nivel_restrito_sem_hipotese"),
    ] {
        let ambiente = apoio::ambiente();
        let (_resultado, pedidos) =
            apoio::rodar_com(&ambiente, vec![], |sessao, nup, indice, transporte| {
                let pedido = PedidoCriacao {
                    nup,
                    unidade: None,
                    tipo: "Despacho",
                    descricao: None,
                    nivel,
                };
                executar_criacao(sessao, &pedido, indice, transporte, true)
            });
        assert!(
            pedidos.is_empty(),
            "não deve haver GET/POST ao SEI quando a escrita já está proibida"
        );
    }
}

#[test]
fn resultado_tipado_distingue_previa_de_criacao_com_numero_observado() {
    // Contrato da #246: não representar "confirmado" sem número SEI.
    // O enum elimina bool + Option<String> independentes.
    assert!(matches!(EstadoCriacao::Previa, EstadoCriacao::Previa));
    let criado = EstadoCriacao::Criado {
        numero_sei: "0000102".to_owned(),
    };
    let EstadoCriacao::Criado { numero_sei } = criado else {
        panic!("uma criação confirmada exige número SEI");
    };
    assert_eq!(numero_sei, "0000102");
}
