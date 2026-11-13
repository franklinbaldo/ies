use super::*;
const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn linha(numero: &str, estado: &str, descricao: &str) -> String {
    format!(
        "<tr><td><input type='checkbox'/></td><td><a href='controlador.php?acao=rel_bloco_protocolo_listar&amp;id_bloco={numero}&amp;infra_hash={HASH}'>{numero}</a></td><td>{descricao}</td><td>{estado}</td><td></td></tr>"
    )
}
fn pagina(linhas: &str, caption: &str) -> String {
    format!(
        "<table id='tblBlocos'><caption>{caption}</caption><tr><th></th><th>Número</th><th>Descrição</th><th>Estado</th><th>Ações</th></tr>{linhas}</table>"
    )
}
#[test]
fn bloco_com_colunas_fora_de_ordem_e_numeros_opacos() {
    // Caso de Todos tests/paginas_reais/menu_blocos.html @323eb01
    let l = ler_lista(
        &crate::tela_html::HtmlServido::de_teste(&pagina(
            &linha("2195", "Gerado", "Teste de Bloco"),
            "Lista de Blocos (1 registros):",
        )),
        TipoBloco::Assinatura,
    )
    .unwrap();
    assert_eq!(l.lista.total_declarado, 1);
    assert_eq!(l.lista.itens[0].numero, "2195");
    assert_eq!(l.lista.itens[0].estado, EstadoDoBloco::Gerado);
    assert_eq!(l.lista.itens[0].descricao, "Teste de Bloco");
    // A tela de listagem não mostra quantidade de itens: o campo não existe.
    assert!(
        !serde_json::to_string(&l.lista)
            .unwrap()
            .contains("quantidade_itens")
    );
    assert!(!serde_json::to_string(&l.lista).unwrap().contains(HASH));
}
#[test]
fn estado_fora_do_conjunto_do_sei_e_recusado() {
    let html = crate::tela_html::HtmlServido::de_teste(&pagina(
        &linha("2195", "Enviado", "X"),
        "Lista (1 registros):",
    ));
    assert_eq!(
        ler_lista(&html, TipoBloco::Interno).err(),
        Some(ErroBloco::EstadoDoBlocoDesconhecido("Enviado".to_owned()))
    );
}
#[test]
fn duplicidade_ou_truncamento_nao_viram_lista_completa() {
    let um = linha("2195", "Gerado", "Exemplo");
    assert_eq!(
        ler_lista(
            &crate::tela_html::HtmlServido::de_teste(&pagina(
                &(um.clone() + &um),
                "Lista (2 registros):"
            )),
            TipoBloco::Interno
        )
        .err()
        .unwrap(),
        ErroBloco::NumeroDoBlocoRepetido
    );
    assert_eq!(
        ler_lista(
            &crate::tela_html::HtmlServido::de_teste(&pagina(&um, "Lista (3 registros):")),
            TipoBloco::Interno
        )
        .err()
        .unwrap(),
        ErroBloco::FaixaDeBlocosIncoerente
    );
}
#[test]
fn faixa_de_pagina_e_observada_nao_inferida() {
    let l = ler_lista(
        &crate::tela_html::HtmlServido::de_teste(&pagina(
            &linha("2195", "Gerado", "X"),
            "Lista (100 registros - 51 a 51):",
        )),
        TipoBloco::Reuniao,
    )
    .unwrap();
    assert_eq!(
        l.lista.faixa_observada,
        Some(Faixa {
            inicio: 51,
            fim: 51
        })
    );
}
#[test]
fn tipo_certo_e_hash_unico_sao_obrigatorios() {
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let base = "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_controlar";
    let href = format!("controlador.php?acao=bloco_interno_listar&infra_hash={HASH}");
    assert!(
        resolver_link(
            &instalacao,
            base,
            &href,
            AcaoDoSei::BlocoInternoListar,
            None
        )
        .is_ok()
    );
    assert_eq!(
        resolver_link(
            &instalacao,
            base,
            &format!("{href}&infra_hash={HASH}"),
            AcaoDoSei::BlocoInternoListar,
            None
        ),
        Err(ErroBloco::ChaveDoLinkDeBloco(
            crate::consulta_do_controlador::ErroParametroDaUrl::ParametroRepetido("infra_hash")
        ))
    );
    assert_eq!(
        resolver_link(
            &instalacao,
            base,
            "https://outra.test/controlador.php?acao=bloco_interno_listar",
            AcaoDoSei::BlocoInternoListar,
            None
        ),
        Err(ErroBloco::LinkDeBlocosNaoReconhecido)
    );
}

#[test]
fn link_de_bloco_com_chave_repetida_ou_par_sem_igual_tem_a_causa() {
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let base = "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_controlar";
    let link = |consulta: String| {
        let href = format!("controlador.php?{consulta}");
        resolver_link(
            &instalacao,
            base,
            &href,
            AcaoDoSei::BlocoInternoListar,
            None,
        )
    };
    assert_eq!(
        link(format!(
            "acao=bloco_interno_listar&acao=bloco_interno_listar&infra_hash={HASH}"
        )),
        Err(ErroBloco::AcaoDoLinkDeBloco(ErroAcaoDaUrl::AcaoRepetida))
    );
    assert_eq!(
        link(format!("acao=bloco_interno_listar&lixo&infra_hash={HASH}")),
        Err(ErroBloco::ConsultaDoLinkDeBloco(
            ErroConsultaDoControlador::ParSemIgual
        ))
    );
    assert_eq!(
        link(format!(
            "acao=bloco_interno_listar&infra_hash={HASH}&id_bloco=5"
        )),
        Err(ErroBloco::LinkDeBlocosNaoReconhecido)
    );
}

#[test]
fn singular_da_legenda_e_aceito_com_um_bloco() {
    let um = linha("2195", "Gerado", "Exemplo");
    let lida = ler_lista(
        &crate::tela_html::HtmlServido::de_teste(&pagina(&um, "Lista (1 registro):")),
        TipoBloco::Interno,
    )
    .unwrap();
    assert_eq!(lida.lista.total_declarado, 1);
    assert_eq!(
        lida.lista.faixa_observada,
        Some(Faixa { inicio: 1, fim: 1 })
    );
    assert_eq!(
        ler_lista(
            &crate::tela_html::HtmlServido::de_teste(&pagina(&um, "Lista (2 registro):")),
            TipoBloco::Interno,
        )
        .err()
        .unwrap(),
        ErroBloco::Legenda(crate::legenda::ErroLegenda::SingularComTotalDiferenteDeUm { total: 2 })
    );
}

#[test]
fn lista_vazia_observada_no_acervo_e_contraprovas() {
    for label in [
        "<label>Nenhum registro encontrado.</label>",
        "<label class=\"infraLabelTabelaNenhumRegistro\">Nenhum registro encontrado.</label>",
    ] {
        let html = crate::tela_html::HtmlServido::de_teste(&format!(
            "<div id=\"divInfraAreaTabela\" class=\"infraAreaTabela\">{label}</div>"
        ));
        let lida = ler_lista(&html, TipoBloco::Interno).unwrap();
        assert_eq!(lida.lista.total_declarado, 0);
        assert_eq!(lida.lista.faixa_observada, None);
        assert_eq!(lida.lista.itens, []);
    }

    // Contraprova: outra mensagem ou conteúdo extra continua sendo TabelaDeBlocosAusente.
    for html_recusado in [
        "<div id=\"divInfraAreaTabela\"><label>Outro texto.</label></div>",
        "<div id=\"divInfraAreaTabela\"><label>Nenhum registro encontrado.</label><span>extra</span></div>",
        "<div><label>Nenhum registro encontrado.</label></div>",
    ] {
        let html = crate::tela_html::HtmlServido::de_teste(html_recusado);
        assert_eq!(
            ler_lista(&html, TipoBloco::Assinatura).err(),
            Some(ErroBloco::TabelaDeBlocosAusente)
        );
    }
}
