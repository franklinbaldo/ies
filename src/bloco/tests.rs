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
        &pagina(
            &linha("2195", "Gerado", "Teste de Bloco"),
            "Lista de Blocos (1 registros):",
        ),
        TipoBloco::Assinatura,
    )
    .unwrap();
    assert_eq!(l.lista.total_declarado, 1);
    assert_eq!(l.lista.itens[0].numero, "2195");
    assert_eq!(l.lista.itens[0].descricao, "Teste de Bloco");
    assert_eq!(l.lista.itens[0].quantidade_itens, None);
    assert!(
        serde_json::to_string(&l.lista)
            .unwrap()
            .contains("\"quantidade_itens\":null")
    );
    assert!(!serde_json::to_string(&l.lista).unwrap().contains(HASH));
}
#[test]
fn duplicidade_ou_truncamento_nao_viram_lista_completa() {
    let um = linha("2195", "Gerado", "Exemplo");
    assert_eq!(
        ler_lista(
            &pagina(&(um.clone() + &um), "Lista (2 registros):"),
            TipoBloco::Interno
        )
        .err()
        .unwrap(),
        ErroBloco::Html(crate::erro_html::ErroHtml::repetido(
            "numero_do_bloco_repetido",
            "o elemento aparece mais de uma vez"
        ))
    );
    assert_eq!(
        ler_lista(&pagina(&um, "Lista (3 registros):"), TipoBloco::Interno)
            .err()
            .unwrap(),
        ErroBloco::FaixaDeBlocosIncoerente
    );
}
#[test]
fn faixa_de_pagina_e_observada_nao_inferida() {
    let l = ler_lista(
        &pagina(
            &linha("2195", "Gerado", "X"),
            "Lista (100 registros - 51 a 51):",
        ),
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
    assert!(resolver_link(&instalacao, base, &href, "bloco_interno_listar", None).is_ok());
    assert_eq!(
        resolver_link(
            &instalacao,
            base,
            &format!("{href}&infra_hash={HASH}"),
            "bloco_interno_listar",
            None
        ),
        Err(ErroBloco::Html(crate::erro_html::ErroHtml::valor(
            "link_de_blocos_nao_reconhecido",
            "link de blocos ausente ou ambíguo na tela"
        )))
    );
    assert_eq!(
        resolver_link(
            &instalacao,
            base,
            "https://outra.test/controlador.php?acao=bloco_interno_listar",
            "bloco_interno_listar",
            None
        ),
        Err(ErroBloco::Html(crate::erro_html::ErroHtml::valor(
            "link_de_blocos_nao_reconhecido",
            "link de blocos ausente ou ambíguo na tela"
        )))
    );
}
