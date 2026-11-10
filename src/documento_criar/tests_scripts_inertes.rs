//! Regressões sintéticas: função de navegação inerte não autoriza POST (#246).
//! Oráculo positivo: Todos `tests/test_escolha_na_lista_010.py` @323eb01.
//! Os casos negativos são adversariais HTML, não capturas de treinamento.
use super::*;

const ESCOLHER: &str = r"function escolher(id) {
    document.getElementById('hdnIdSerie').value = id;
    document.getElementById('frmDocumentoEscolherTipo').submit();
}";

const EXIBIR: &str = r"function exibirSeries(tipo) {
    document.getElementById('hdnFiltroSerie').value = tipo;
    document.getElementById('frmDocumentoEscolherTipo').submit();
}";

fn script_inerte(codigo: &str, forma: usize) -> String {
    match forma {
        0 => format!("<template><script>{codigo}</script></template>"),
        1 => format!("<script type='application/json'>{codigo}</script>"),
        2 => format!("<!-- <script>{codigo}</script> -->"),
        _ => unreachable!(),
    }
}

#[test]
fn escolher_ignora_codigo_em_template_json_ou_comentario_html() {
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let tabela = r##"<table id="tblSeries"><tr><td>
        <a href="#" onclick="escolher(291)">Despacho</a>
        </td></tr></table>"##;
    for forma in 0..3 {
        let tela = format!("{}{}", script_inerte(ESCOLHER, forma), tabela);
        assert_eq!(
            escolha_do_tipo(&instalacao, &tela, "4864", "Despacho"),
            Err(ErroCriar::LinkDoTipoNaoNavegavel),
            "forma inerte {forma} não pode autorizar navegação POST"
        );
    }
    let ativa = format!("<script>{ESCOLHER}</script>{tabela}");
    assert_eq!(
        escolha_do_tipo(&instalacao, &ativa, "4864", "Despacho"),
        Ok(EscolhaDoDocumento::Formulario("291".to_owned()))
    );
}

#[test]
fn filtro_exibir_todos_ignora_script_inerte() {
    let botoes = r#"<a id="ancExibirSeries" href="javascript:void(0);"
        onclick="exibirSeries('T');">Todos</a>
        <table id="tblSeries"></table>"#;
    for forma in 0..3 {
        let tela = format!("{}{}", script_inerte(EXIBIR, forma), botoes);
        assert_eq!(
            filtro_de_todos(&tela),
            Err(ErroCriar::LinkDoTipoNaoNavegavel),
            "forma inerte {forma} não pode autorizar POST do filtro"
        );
    }
    let ativa = format!("<script>{EXIBIR}</script>{botoes}");
    assert_eq!(filtro_de_todos(&ativa), Ok(true));
}

#[test]
fn comentario_string_e_bloco_inalcancavel_nao_provam_navegacao() {
    // Caso negativo sintético derivado das funções reais do Todos @323eb01:
    // texto dentro de JS ativo não implica que a função seja alcançável.
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let tabela = r##"<table id="tblSeries"><tr><td>
        <a href="#" onclick="escolher(291)">Despacho</a>
        </td></tr></table>"##;
    let filtro = r#"<a id="ancExibirSeries" href="javascript:void(0);"
        onclick="exibirSeries('T');">Todos</a>
        <table id="tblSeries"></table>"#;
    for (forma, montar) in [
        (0, format!("/* {ESCOLHER} */")),
        (1, format!("const exemplo = `{ESCOLHER}`;")),
        (2, format!("if (false) {{ {ESCOLHER} }}")),
    ] {
        let tela = format!("<script>{montar}</script>{tabela}");
        assert_eq!(
            escolha_do_tipo(&instalacao, &tela, "4864", "Despacho"),
            Err(ErroCriar::LinkDoTipoNaoNavegavel),
            "função escolher inerte em JS ativo forma {forma}"
        );
    }
    for (forma, montar) in [
        (0, format!("// {EXIBIR}")),
        (1, format!("const exemplo = `{EXIBIR}`;")),
        (2, format!("if (false) {{ {EXIBIR} }}")),
    ] {
        let tela = format!("<script>{montar}</script>{filtro}");
        assert_eq!(
            filtro_de_todos(&tela),
            Err(ErroCriar::LinkDoTipoNaoNavegavel),
            "função exibirSeries inerte em JS ativo forma {forma}"
        );
    }
}

#[test]
fn expressao_nomeada_nao_publica_funcao_global_de_navegacao() {
    // Regressão adversarial #246: nome de expressão function é local e
    // não define a função global chamada pelo onclick do SEI.
    // Oráculo positivo: Todos tests/test_escolha_na_lista_010.py @323eb01.
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let tabela = r##"<table id="tblSeries"><tr><td>
        <a href="#" onclick="escolher(291)">Despacho</a>
        </td></tr></table>"##;
    let filtro = r#"<a id="ancExibirSeries" href="javascript:void(0);"
        onclick="exibirSeries('T');">Todos</a><table id="tblSeries"></table>"#;
    for expressao in [
        format!("const interna = {ESCOLHER};"),
        format!("const interna = ({ESCOLHER});"),
    ] {
        let html = format!("<script>{expressao}</script>{tabela}");
        assert_eq!(
            escolha_do_tipo(&instalacao, &html, "4864", "Despacho"),
            Err(ErroCriar::LinkDoTipoNaoNavegavel),
            "expressão não-global não autoriza POST de escolha"
        );
    }
    for expressao in [
        format!("const interna = {EXIBIR};"),
        format!("const interna = ({EXIBIR});"),
    ] {
        let html = format!("<script>{expressao}</script>{filtro}");
        assert_eq!(
            filtro_de_todos(&html),
            Err(ErroCriar::LinkDoTipoNaoNavegavel),
            "expressão não-global não autoriza POST de filtro"
        );
    }
    // Uma declaração real após outro comando completo segue aceita.
    assert_eq!(
        escolha_do_tipo(
            &instalacao,
            &format!("<script>const marcador = 1; {ESCOLHER}</script>{tabela}"),
            "4864",
            "Despacho",
        ),
        Ok(EscolhaDoDocumento::Formulario("291".to_owned()))
    );
}

#[test]
fn expressao_na_lista_servida_recusa_post_com_orcamento_exato() {
    // Regressão end-to-end para a guarda do POST de navegação: sem função
    // global publicada, quatro GETs de prévia; nenhum POST de filtro/escolha.
    use super::fluxo::{ErroPreviaCriacao, PedidoCriacao, executar_criacao};
    use crate::processo_concluir::tests_apoio as apoio;
    use crate::propriedades_processo::tests_apoio::acao_da_barra;
    let link = format!(
        "controlador.php?acao=documento_escolher_tipo&id_procedimento=10&infra_hash={}",
        "e".repeat(64),
    );
    let arvore = apoio::arvore(&[apoio::UNIDADE], &[acao_da_barra(&link)]);
    let lista = format!(
        r##"<script>const interna = {ESCOLHER};</script><table id="tblSeries">
        <tr><td><a href="#" onclick="escolher(291)">Despacho</a></td></tr>
        </table>"##,
    );
    let mut respostas = apoio::abertura(arvore);
    respostas.push(apoio::ok(lista));
    let ambiente = apoio::ambiente();
    let (resultado, pedidos) =
        apoio::rodar_com(&ambiente, respostas, |sessao, nup, indice, rede| {
            let pedido = PedidoCriacao {
                nup,
                unidade: None,
                tipo: "Despacho",
                descricao: Some("Ensaio"),
                nivel: Some("publico"),
            };
            executar_criacao(sessao, &pedido, indice, rede, true)
        });
    assert!(matches!(
        resultado,
        Err(ErroPreviaCriacao::Tipo(ErroCriar::LinkDoTipoNaoNavegavel))
    ));
    let mut esperados = apoio::pedidos_da_abertura();
    esperados.push(apoio::get(&format!("{}/{}", apoio::ORIGEM, link)));
    assert_eq!(pedidos, esperados, "quatro GETs e zero POSTs");
}

#[test]
fn controles_de_tipo_dentro_de_template_nao_sao_oferta() {
    // Cenário HTML adversarial sintético, não atribuído ao SEI real:
    // um link em template não está disponível para clicar ou selecionar.
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let href = format!(
        "controlador.php?acao=documento_gerar&id_procedimento=4864&infra_hash={}",
        "a".repeat(64)
    );
    let visivel = format!(r#"<a href="{href}">Despacho</a>"#);
    let lista = format!(
        r#"<table id="tblSeries"><tr><td>{visivel}<template>{visivel}</template></td></tr></table>"#
    );
    assert_eq!(tipos_oferecidos(&lista).unwrap(), ["Despacho"]);
    assert_eq!(
        escolha_do_tipo(&instalacao, &lista, "4864", "Despacho"),
        Ok(EscolhaDoDocumento::Link(format!(
            "https://sei.exemplo.gov.br/sei/{href}"
        )))
    );
    let apenas_inerte = format!(
        r#"<table id="tblSeries"><tr><td><template>{visivel}</template></td></tr></table>"#
    );
    assert_eq!(
        tipos_oferecidos(&apenas_inerte).unwrap(),
        Vec::<String>::new()
    );
    assert_eq!(
        escolha_do_tipo(&instalacao, &apenas_inerte, "4864", "Despacho"),
        Err(ErroCriar::TipoAusente)
    );
}

#[test]
fn botao_exibir_todos_em_template_nao_autoriza_filtro() {
    // O script é ativo, mas o único acionador está num fragmento inerte.
    // Nenhum POST de filtro deve ser derivado desse HTML.
    let botao = r#"<a id="ancExibirSeries" href="javascript:void(0);" onclick="exibirSeries('T');">Todos</a>"#;
    let html = format!(
        "<script>{EXIBIR}</script><template>{botao}</template><table id=\"tblSeries\"></table>"
    );
    assert_eq!(filtro_de_todos(&html), Ok(false));
}

#[test]
fn tipos_em_controles_hidden_nao_sao_oferecidos() {
    // Adversarial sintético: hidden no próprio link ou em ancestral
    // retira o tipo da oferta; a cópia visível segue selecionável.
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let href = format!(
        "controlador.php?acao=documento_gerar&id_procedimento=4864&infra_hash={}",
        "a".repeat(64)
    );
    let visivel = format!(r#"<a href="{href}">Despacho</a>"#);
    let oculto = format!(r#"<a hidden href="{href}">Despacho</a>"#);
    for inerte in [&oculto, &format!("<div hidden>{visivel}</div>")] {
        let lista = format!(r#"<table id="tblSeries"><tr><td>{visivel}{inerte}</td></tr></table>"#);
        assert_eq!(tipos_oferecidos(&lista).unwrap(), ["Despacho"]);
        assert_eq!(
            escolha_do_tipo(&instalacao, &lista, "4864", "Despacho"),
            Ok(EscolhaDoDocumento::Link(format!(
                "https://sei.exemplo.gov.br/sei/{href}"
            )))
        );
        let sem_visivel = format!(r#"<table id="tblSeries"><tr><td>{inerte}</td></tr></table>"#);
        assert_eq!(
            tipos_oferecidos(&sem_visivel).unwrap(),
            Vec::<String>::new()
        );
        assert_eq!(
            escolha_do_tipo(&instalacao, &sem_visivel, "4864", "Despacho"),
            Err(ErroCriar::TipoAusente)
        );
    }
}

#[test]
fn exibir_todos_hidden_nao_autoriza_filtro() {
    // Script real isolado não oferece o link oculto; não emitir POST.
    let botao = r#"<a id="ancExibirSeries" href="javascript:void(0);" onclick="exibirSeries('T');">Todos</a>"#;
    let botao_oculto = r#"<a hidden id="ancExibirSeries" href="javascript:void(0);" onclick="exibirSeries('T');">Todos</a>"#;
    for inerte in [
        format!("<div hidden>{botao}</div>"),
        botao_oculto.to_owned(),
    ] {
        let tela = format!("<script>{EXIBIR}</script>{inerte}<table id=\"tblSeries\"></table>");
        assert_eq!(filtro_de_todos(&tela), Ok(false));
    }
}

#[test]
fn tipo_interno_nao_e_ambiguo_por_link_externo_homonimo() {
    // Regressao tecnica do BDD-246-03: a CLI so oferece tipos internos.
    // Uma entrada externa com o mesmo rotulo nao e segunda opcao nativa.
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let href = format!(
        "controlador.php?acao=documento_gerar&id_procedimento=4864&infra_hash={}",
        "a".repeat(64)
    );
    let html = format!(
        r#"<table id="tblSeries"><tr><td>
        <a href="{href}">Despacho</a>
        <a href="controlador.php?acao=documento_receber">Despacho</a>
        </td></tr></table>"#
    );
    assert_eq!(tipos_oferecidos(&html).unwrap(), ["Despacho"]);
    assert_eq!(
        escolha_do_tipo(&instalacao, &html, "4864", "Despacho"),
        Ok(EscolhaDoDocumento::Link(format!(
            "https://sei.exemplo.gov.br/sei/{href}"
        )))
    );
}
