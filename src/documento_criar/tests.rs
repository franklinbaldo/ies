//! Casos sintéticos do cadastro e da reconciliação de documento interno.
use super::*;
use crate::arvore_processo::{Cancelamento, NaturezaDoDocumento};
use crate::tela_html::HtmlServido;
fn doc(id: &str, numero: Option<&str>, rotulo: &str) -> DocumentoNaArvore {
    DocumentoNaArvore {
        id: id.into(),
        rotulo: rotulo.into(),
        numero_sei: numero.map(str::to_owned),
        pasta: None,
        natureza: NaturezaDoDocumento {
            icone: "documento_interno".into(),
            cancelamento: Cancelamento::Ativo,
            dica: None,
        },
    }
}
#[test]
fn lista_tipo_ausente_ambiguo_e_externo() {
    // Caso de Todos src/todos/documentos.py::_tipos_da_tela @323eb01
    let html = HtmlServido::de_teste(
        r#"<table id="tblSeries"><tr><td>
        <a href="controlador.php?acao=documento_gerar">Despacho</a>
        <a href="controlador.php?acao=documento_receber">Externo</a>
        <a href="javascript:void(0);" onclick="escolher(3);">Ofício</a>
        </td></tr></table>"#,
    );
    let tipos = tipos_oferecidos(&html).unwrap();
    assert_eq!(tipos, ["Despacho", "Ofício"]);
    assert_eq!(conferir_tipo("Despacho", &tipos), Ok(()));
    assert_eq!(
        conferir_tipo("Parecer", &tipos),
        Err(ErroCriar::TipoAusente)
    );
    assert_eq!(
        conferir_tipo("Despacho", &["Despacho".into(), "Despacho".into()]),
        Err(ErroCriar::TipoAmbiguo)
    );
}
#[test]
fn lista_fora_da_forma_falha_fechado() {
    assert_eq!(
        tipos_oferecidos(&HtmlServido::de_teste("<html></html>")),
        Err(ErroCriar::ListaForaDaForma)
    );
}
#[test]
fn novo_unico_com_numero_confere() {
    // Caso de Todos src/todos/documentos.py::_conferir_criacao @323eb01
    let antes = [doc("1", Some("000001"), "Ofício 000001")];
    let depois = [
        antes[0].clone(),
        doc("2", Some("000002"), "Despacho 000002"),
    ];
    assert_eq!(
        conferir_criacao(&antes, &depois, "Despacho"),
        Ok("000002".into())
    );
}
#[test]
fn nenhum_novo_e_distinto_de_nao_confirmado() {
    let antes = [doc("1", Some("000001"), "Ofício 000001")];
    assert_eq!(
        conferir_criacao(&antes, &antes, "Despacho"),
        Err(ErroCriar::NaoAplicado)
    );
    let depois = [
        antes[0].clone(),
        doc("2", Some("000002"), "Despacho 000002"),
        doc("3", Some("000003"), "Despacho 000003"),
    ];
    assert_eq!(
        conferir_criacao(&antes, &depois, "Despacho"),
        Err(ErroCriar::NaoConfirmado)
    );
    let sem_numero = [antes[0].clone(), doc("2", None, "Despacho")];
    assert_eq!(
        conferir_criacao(&antes, &sem_numero, "Despacho"),
        Err(ErroCriar::NaoConfirmado)
    );
}
#[test]
fn tipo_errado_e_documento_antigo_alterado_sao_incertos() {
    let antes = [doc("1", Some("000001"), "Ofício 000001")];
    let tipo = [antes[0].clone(), doc("2", Some("000002"), "Parecer 000002")];
    assert_eq!(
        conferir_criacao(&antes, &tipo, "Despacho"),
        Err(ErroCriar::NaoConfirmado)
    );
    let antigo = [
        doc("1", Some("000001"), "Ofício editado 000001"),
        doc("2", Some("000002"), "Despacho 000002"),
    ];
    assert_eq!(
        conferir_criacao(&antes, &antigo, "Despacho"),
        Err(ErroCriar::NaoConfirmado)
    );
}

#[test]
fn criacao_recusa_arvore_inicial_sem_identidade_unica() {
    // Um snapshot inicial inconsistente não pode provar escrita única.
    let original = doc("1", Some("000001"), "Ofício 000001");
    let antes = [original.clone(), original.clone()];
    let depois = [original, doc("2", Some("000002"), "Despacho 000002")];
    assert_eq!(
        conferir_criacao(&antes, &depois, "Despacho"),
        Err(ErroCriar::NaoConfirmado)
    );
}

#[test]
fn criacao_recusa_identidade_nova_vazia() {
    // Um nó sem id não pode estabelecer unicidade no SEI.
    let depois = [doc("", Some("000002"), "Despacho 000002")];
    assert_eq!(
        conferir_criacao(&[], &depois, "Despacho"),
        Err(ErroCriar::NaoConfirmado)
    );
}

#[test]
fn link_direto_confere_acao_processo_e_origem_sem_executar_js() {
    // Caso de Todos src/todos/documentos.py::_tipos_da_tela @323eb01
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let href = format!(
        "controlador.php?acao=documento_gerar&id_procedimento=10&infra_hash={}",
        "a".repeat(64)
    );
    let html = HtmlServido::de_teste(&format!(
        "<table id=\"tblSeries\"><tr><td><a href=\"{href}\">Despacho</a></td></tr></table>"
    ));
    assert_eq!(
        escolha_do_tipo(&instalacao, &html, "10", "Despacho"),
        Ok(EscolhaDoDocumento::Link(format!(
            "https://sei.exemplo.gov.br/sei/{href}"
        )))
    );
    assert!(matches!(
        escolha_do_tipo(&instalacao, &html, "11", "Despacho"),
        Err(ErroCriar::EnderecoDoTipoNaoReconhecido(_))
    ));
    let script = HtmlServido::de_teste(
        r#"<table id="tblSeries"><tr><td>
        <a href="javascript:void(0);" onclick="escolher(3);">Despacho</a>
        </td></tr></table>"#,
    );
    assert_eq!(
        escolha_do_tipo(&instalacao, &script, "10", "Despacho"),
        Err(ErroCriar::NavegacaoNaoComprovadaPorScript(
            super::guarda_navegacao::ErroGuardaDeNavegacao::FuncaoNaoDeclaradaEmScriptAtivo
        ))
    );
}

#[test]
fn cadastro_recusa_nivel_com_valor_radio_diferente_do_observado() {
    // O número do nível vem da tela, não pode ser escolhido por suposição.
    let html = HtmlServido::de_teste(
        r#"<form id="frmDocumentoCadastro" method="post">
        <label id="lblSerieTitulo">Despacho</label>
        <input type="radio" id="optPublico" name="rdoNivelAcesso" value="9">
        </form>"#,
    );
    assert_eq!(
        conferir_cadastro(&html, "Despacho", None, Some("publico")),
        Err(ErroCriar::NivelNaoOferecido)
    );
}

#[test]
fn cadastro_so_aceita_tipo_e_controles_publicados_habilitados() {
    // Caso de Todos tests/paginas_reais/documento_gerar.html @323eb01
    let html = r#"<form id="frmDocumentoCadastro" method="post">
        <label id="lblSerieTitulo">Despacho</label>
        <input type="text" id="txtDescricao" name="txtDescricao">
        <input type="radio" id="optPublico" name="rdoNivelAcesso" value="0">
        <input type="radio" id="optRestrito" name="rdoNivelAcesso" value="1" disabled="disabled">
        </form>"#;
    assert_eq!(
        conferir_cadastro(
            &HtmlServido::de_teste(html),
            "Despacho",
            Some("Ensaio"),
            Some("publico")
        ),
        Ok(())
    );
    assert_eq!(
        conferir_cadastro(&HtmlServido::de_teste(html), "Ofício", None, None),
        Err(ErroCriar::TipoDoCadastroDiferente)
    );
    assert_eq!(
        conferir_cadastro(
            &HtmlServido::de_teste(html),
            "Despacho",
            None,
            Some("restrito")
        ),
        Err(ErroCriar::NivelNaoOferecido)
    );
    assert_eq!(
        conferir_cadastro(
            &HtmlServido::de_teste(html),
            "Despacho",
            None,
            Some("sigiloso")
        ),
        Err(ErroCriar::NivelNaoOferecido)
    );
    let sem_campo = HtmlServido::de_teste(&html.replace("id=\"txtDescricao\"", "id=\"outro\""));
    assert_eq!(
        conferir_cadastro(&sem_campo, "Despacho", Some("texto"), None),
        Err(ErroCriar::DescricaoNaoOferecida)
    );
    assert_eq!(
        conferir_cadastro(
            &HtmlServido::de_teste("<html></html>"),
            "Despacho",
            None,
            None
        ),
        Err(ErroCriar::CadastroForaDaForma)
    );
}

#[test]
fn controles_de_cadastro_inertes_ou_fora_do_formulario_nao_autorizam_escrita() {
    // Cenários adversariais sintéticos: a presença de um id na página
    // não prova que o formulário de cadastro oferece esse controle.
    let abertura = r#"<form id="frmDocumentoCadastro" method="post">
        <label id="lblSerieTitulo">Despacho</label>"#;
    let publico = r#"<input type="radio" id="optPublico" name="rdoNivelAcesso" value="0">"#;
    let descricao = r#"<input id="txtDescricao" name="txtDescricao" type="text">"#;

    let dentro_de_template = HtmlServido::de_teste(&format!(
        "{abertura}{publico}<template>{descricao}</template></form>"
    ));
    assert_eq!(
        conferir_cadastro(
            &dentro_de_template,
            "Despacho",
            Some("Prova"),
            Some("publico")
        ),
        Err(ErroCriar::DescricaoNaoOferecida)
    );

    let fora_do_formulario =
        HtmlServido::de_teste(&format!("{descricao}{abertura}{publico}</form>"));
    assert_eq!(
        conferir_cadastro(
            &fora_do_formulario,
            "Despacho",
            Some("Prova"),
            Some("publico")
        ),
        Err(ErroCriar::DescricaoNaoOferecida)
    );

    let radio_desabilitado =
        format!("{abertura}<fieldset disabled>{publico}</fieldset>{descricao}</form>");
    assert_eq!(
        conferir_cadastro(
            &HtmlServido::de_teste(&radio_desabilitado),
            "Despacho",
            Some("Prova"),
            Some("publico")
        ),
        Err(ErroCriar::NivelNaoOferecido)
    );
}

#[test]
fn tipo_41_requer_escolha_por_formulario_sem_executar_js() {
    // Caso de Todos tests/test_escolha_na_lista_010.py @323eb01 (UFRJ 4.1.5).
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let html = r##"<html><head><script>
        function escolher(id) {
            document.getElementById('hdnIdSerie').value = id;
            document.getElementById('frmDocumentoEscolherTipo').submit();
        }
        </script></head><body>
        <form id="frmDocumentoEscolherTipo" method="post">
            <input type="hidden" id="hdnIdSerie" name="hdnIdSerie" value="">
        </form>
        <table id="tblSeries"><tr><td>
            <a href="#" onclick="escolher(291)">Despacho</a>
            <a href="#" onclick="escolher(-1)">Externo</a>
        </td></tr></table></body></html>"##;
    assert_eq!(
        tipos_oferecidos(&HtmlServido::de_teste(html)).unwrap(),
        ["Despacho"]
    );
    assert_eq!(
        escolha_do_tipo(
            &instalacao,
            &HtmlServido::de_teste(html),
            "4864",
            "Despacho"
        ),
        Ok(EscolhaDoDocumento::Formulario("291".into()))
    );
    assert_eq!(
        escolha_do_tipo(&instalacao, &HtmlServido::de_teste(html), "4864", "Externo"),
        Err(ErroCriar::TipoAusente)
    );
}

#[test]
fn escolha_41_nao_confia_em_onclick_sem_prova_do_script() {
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let original = r##"<script>
    function escolher(id) {
        document.getElementById('hdnIdSerie').value = id;
        document.getElementById('frmDocumentoEscolherTipo').submit();
    }</script>
    <table id="tblSeries"><tr><td>
        <a href="#" onclick="escolher(291)">Despacho</a>
    </td></tr></table>"##;
    let com_efeito_extra = HtmlServido::de_teste(&original.replace(
        "document.getElementById('frmDocumentoEscolherTipo').submit();",
        "alert('extra'); document.getElementById('frmDocumentoEscolherTipo').submit();",
    ));
    assert_eq!(
        escolha_do_tipo(&instalacao, &com_efeito_extra, "4864", "Despacho"),
        Err(ErroCriar::NavegacaoNaoComprovadaPorScript(
            super::guarda_navegacao::ErroGuardaDeNavegacao::ScriptForaDaFormaObservada
        ))
    );
    assert_eq!(
        escolha_do_tipo(
            &instalacao,
            &HtmlServido::de_teste(&original.replace("function escolher", "function outro")),
            "4864",
            "Despacho"
        ),
        Err(ErroCriar::NavegacaoNaoComprovadaPorScript(
            super::guarda_navegacao::ErroGuardaDeNavegacao::FuncaoNaoDeclaradaEmScriptAtivo
        ))
    );
    assert_eq!(
        escolha_do_tipo(
            &instalacao,
            &HtmlServido::de_teste(&original.replace("escolher(291)", "escolher(291); excluir()")),
            "4864",
            "Despacho"
        ),
        Err(ErroCriar::LinkDoTipoNaoNavegavel)
    );
}

#[test]
fn redirecionamento_da_escolha_valida_origem_processo_e_assinatura() {
    // Caso de Todos tests/test_escolha_na_lista_010.py @323eb01:
    // POST documento_escolher_tipo -> 302 documento_gerar, sem seguir automaticamente.
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let link = format!(
        "controlador.php?acao=documento_gerar&id_procedimento=4864&infra_hash={}",
        "a".repeat(64)
    );
    let esperado = format!("https://sei.exemplo.gov.br/sei/{link}");
    assert_eq!(
        cadastro_redirecionado(&instalacao, &link, "4864"),
        Ok(esperado.clone())
    );
    assert_eq!(
        cadastro_redirecionado(&instalacao, &esperado, "4864"),
        Ok(esperado)
    );
    assert!(matches!(
        cadastro_redirecionado(&instalacao, &link, "4865"),
        Err(ErroCriar::EnderecoDoTipoNaoReconhecido(_))
    ));
    let externo = format!("https://host-malicioso.example/sei/{link}");
    assert!(matches!(
        cadastro_redirecionado(&instalacao, &externo, "4864"),
        Err(ErroCriar::EnderecoDoTipoNaoReconhecido(_))
    ));
    let sem_assinatura = "controlador.php?acao=documento_gerar&id_procedimento=4864";
    assert!(matches!(
        cadastro_redirecionado(&instalacao, sem_assinatura, "4864"),
        Err(ErroCriar::EnderecoDoTipoNaoReconhecido(_))
    ));
    let duplicado = format!("{link}&infra_hash={}", "b".repeat(64));
    assert!(matches!(
        cadastro_redirecionado(&instalacao, &duplicado, "4864"),
        Err(ErroCriar::EnderecoDoTipoNaoReconhecido(_))
    ));
}

#[test]
fn exibir_todos_so_por_acao_publicada_e_sem_js_executado() {
    // Caso de Todos src/todos/documentos.py::_escolha_do_tipo @323eb01.
    // Tela 4.0: exibirSeries('T') grava hdnFiltroSerie e submete a lista.
    let tela = r#"<html><head><script>
    function exibirSeries(tipo) {
        document.getElementById('hdnFiltroSerie').value = tipo;
        document.getElementById('frmDocumentoEscolherTipo').submit();
    }
    </script></head><body>
    <form id="frmDocumentoEscolherTipo" method="post">
       <input type="hidden" name="hdnFiltroSerie" id="hdnFiltroSerie" value="U">
    </form>
    <a id="ancExibirSeries" href="javascript:void(0);" onclick="exibirSeries('T');">Todos</a>
    <table id="tblSeries"><tr><td>
       <a href="controlador.php?acao=documento_gerar">Despacho</a>
    </td></tr></table></body></html>"#;
    assert_eq!(filtro_de_todos(&HtmlServido::de_teste(tela)), Ok(true));
    assert_eq!(
        filtro_de_todos(&HtmlServido::de_teste(
            &tela.replace("ancExibirSeries", "outro")
        )),
        Ok(false)
    );
    assert_eq!(
        filtro_de_todos(&HtmlServido::de_teste(
            &tela.replace("function exibirSeries", "function sem_prova")
        )),
        Err(ErroCriar::NavegacaoNaoComprovadaPorScript(
            super::guarda_navegacao::ErroGuardaDeNavegacao::FuncaoNaoDeclaradaEmScriptAtivo
        ))
    );
    assert_eq!(
        filtro_de_todos(&HtmlServido::de_teste(&tela.replace(
            "document.getElementById('frmDocumentoEscolherTipo').submit();",
            "alert('extra'); document.getElementById('frmDocumentoEscolherTipo').submit();",
        ))),
        Err(ErroCriar::NavegacaoNaoComprovadaPorScript(
            super::guarda_navegacao::ErroGuardaDeNavegacao::ScriptForaDaFormaObservada
        ))
    );
}

#[test]
fn link_direto_do_oraculo_aceita_duas_identidades_iguais_sem_refazer_assinatura() {
    // Caso Todos tests/paginas_reais/documento_escolher_tipo.html @323eb01:
    // o href de Despacho publica duas vezes id_procedimento=50693.
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let href = format!(
        "controlador.php?acao=documento_gerar&acao_origem=documento_escolher_tipo&id_procedimento=50693&id_serie=291&id_procedimento=50693&infra_hash={}",
        "a".repeat(64)
    );
    let html = HtmlServido::de_teste(&format!(
        "<table id=\"tblSeries\"><tr><td><a href=\"{href}\">Despacho</a></td></tr></table>"
    ));
    assert_eq!(
        escolha_do_tipo(&instalacao, &html, "50693", "Despacho"),
        Ok(EscolhaDoDocumento::Link(format!(
            "https://sei.exemplo.gov.br/sei/{href}"
        )))
    );
    let divergente = href.replacen(
        "&id_procedimento=50693&infra_hash=",
        "&id_procedimento=50000&infra_hash=",
        1,
    );
    let html_divergente = HtmlServido::de_teste(&format!(
        "<table id=\"tblSeries\"><tr><td><a href=\"{divergente}\">Despacho</a></td></tr></table>"
    ));
    assert!(matches!(
        escolha_do_tipo(&instalacao, &html_divergente, "50693", "Despacho"),
        Err(ErroCriar::EnderecoDoTipoNaoReconhecido(_))
    ));
    let triplo = href.replace("&infra_hash=", "&id_procedimento=50693&infra_hash=");
    let html_triplo = HtmlServido::de_teste(&format!(
        "<table id=\"tblSeries\"><tr><td><a href=\"{triplo}\">Despacho</a></td></tr></table>"
    ));
    assert!(matches!(
        escolha_do_tipo(&instalacao, &html_triplo, "50693", "Despacho"),
        Err(ErroCriar::EnderecoDoTipoNaoReconhecido(_))
    ));
}
