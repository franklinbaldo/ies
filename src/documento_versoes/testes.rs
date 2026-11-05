use super::*;
fn pagina(numeros: &[u32], ids: &[&str]) -> String {
    use std::fmt::Write as _;
    let mut linhas = String::new();
    for (&n, &id) in numeros.iter().zip(ids) {
        let _ = write!(
            linhas,
            "<tr><td><input type=\"checkbox\" /></td><td>{n}</td><td><a>Ana</a></td><td><a>PPI</a></td><td>31/10/2026</td><td><a href=\"controlador.php?acao=documento_visualizar&amp;id_documento={id}&amp;versao={n}&amp;infra_hash={}\">Ver</a></td></tr>",
            "a".repeat(64)
        );
    }
    format!(
        "<table class=\"infraTable\"><caption>Lista de Versões ({} registros):</caption><tr><th></th><th>Versão</th><th>Usuário</th><th>Unidade</th><th>Última Modificação</th><th>Ações</th></tr>{linhas}</table>",
        numeros.len()
    )
}
/// A forma servida nas sete instalações (`liberacoes-e-versoes-leitura-*`):
/// `id_documento` duas vezes com o mesmo valor, `id_procedimento`, e nas
/// versões anteriores a âncora "Recuperar Versão" (`href="#ID-N"`, link
/// no `onclick`, nunca seguido).
pub(crate) fn pagina_servida(id: &str, outro_id: &str, numeros: &[u32]) -> String {
    use std::fmt::Write as _;
    let hash = "d".repeat(64);
    let atual = numeros.iter().copied().max().unwrap_or(0);
    let mut linhas = String::new();
    for &n in numeros {
        let recuperar = if n == atual {
            String::new()
        } else {
            format!(
                "<a href=\"#ID-{n}\" onclick=\"acaoRecuperar('{n}','N','controlador.php?acao=documento_versao_recuperar&versao={n}&id_documento={id}&infra_hash={hash}');\" tabindex=\"1006\"><img title=\"Recuperar Versão\" /></a>&nbsp;"
            )
        };
        let _ = write!(
            linhas,
            "<tr class=\"infraTrClara\"><td valign=\"center\"><a id=\"lnkInfraID-{n}\" name=\"ID-{n}\"></a><input class=\"infraCheckbox\" type=\"checkbox\" value=\"{n}\"/></td><td align=\"center\">{n}</td>\n<td align=\"center\"><a alt=\"Usuário Teste\" title=\"Usuário Teste\" class=\"ancoraSigla\">USUARIO</a></td>\n<td align=\"center\"><a alt=\"Unidade de Teste 1\" title=\"Unidade de Teste 1\" class=\"ancoraSigla\">TESTE</a></td><td align=\"center\">20/10/2026 14:07:0{n}</td><td align=\"center\"><a target=\"_blank\" onclick=\"infraLimparFormatarTrAcessada(this.parentNode.parentNode);\" href=\"controlador.php?acao=documento_visualizar&acao_origem=documento_versao_listar&acao_retorno=documento_versao_listar&versao={n}&id_documento={id}&arvore=1&id_procedimento=10&id_documento={outro_id}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={hash}\" tabindex=\"1003\"><img src=\"svg/documento_interno.svg?25\" title=\"Visualizar Versão\" /></a>&nbsp;{recuperar}</td></tr>\n"
        );
    }
    format!(
        "<html><body><form><div><table width=\"99%\" class=\"infraTable\" summary=\"Tabela de Versões.\">\n<caption class=\"infraCaption\">Lista de Versões ({} registros):</caption><tr><th class=\"infraTh\" width=\"1%\"><a href=\"javascript:void(0);\" id=\"lnkInfraCheck\"><img title=\"Selecionar Tudo\" alt=\"Selecionar Tudo\" class=\"infraImg\"/></a></th>\n<th class=\"infraTh\">Versão</th>\n<th class=\"infraTh\">Usuário</th>\n<th class=\"infraTh\">Unidade</th>\n<th class=\"infraTh\">Última Modificação</th>\n<th class=\"infraTh\">Ações</th>\n</tr>\n{linhas}</table></div><input type=\"hidden\" id=\"hdnInfraNroItens\" value=\"{}\"/></form></body></html>",
        numeros.len(),
        numeros.len()
    )
}

#[test]
fn forma_servida_com_id_repetido_e_ancora_de_recuperar_e_lida() {
    let doc = NumeroDocumento::novo("0007").unwrap();
    let h = interpretar_historico(&pagina_servida("42", "42", &[2, 1]), &doc, "42", None).unwrap();
    assert_eq!(h.atual().unwrap().numero(), 2);
    assert_eq!(h.len(), 2);
    let v1 = h.selecionar(VersaoDocumento::nova(1).unwrap()).unwrap();
    assert!(v1.link_privado().contains("versao=1&"));
    assert!(!v1.link_privado().contains("documento_versao_recuperar"));
    assert_eq!(v1.usuario, "USUARIO");
    assert_eq!(v1.unidade, "TESTE");
}

#[test]
fn id_repetido_com_valores_diferentes_e_recusado() {
    let doc = NumeroDocumento::novo("0007").unwrap();
    assert_eq!(
        interpretar_historico(&pagina_servida("42", "43", &[1]), &doc, "42", None),
        Err(ErroVersao::LinkInseguro)
    );
    // Uma âncora a mais que não é fragmento continua ambígua.
    let extra = pagina_servida("42", "42", &[1]).replace(
        "&nbsp;</td></tr>",
        "<a href=\"controlador.php?acao=outra\">x</a></td></tr>",
    );
    assert_eq!(
        interpretar_historico(&extra, &doc, "42", None),
        Err(ErroVersao::Html(crate::erro_html::ErroHtml::ausente(
            "link_versao_ausente_ou_ambiguo",
            "a tela não tem o elemento observado"
        )))
    );
}

#[test]
fn booleano_antes_de_class_e_href_nao_esconde_versoes() {
    let doc = NumeroDocumento::novo("0007").unwrap();
    let html =
        crate::html_atributos::com_booleano_antes_dos_atributos(&pagina(&[1, 4], &["42", "42"]));
    let h = interpretar_historico(&html, &doc, "42", Some(2)).unwrap();
    assert_eq!(h.atual().unwrap().numero(), 4);
}
#[test]
fn atual_e_nao_contiguas() {
    let doc = NumeroDocumento::novo("0007").unwrap();
    let h = interpretar_historico(&pagina(&[1, 4], &["42", "42"]), &doc, "42", Some(2)).unwrap();
    assert_eq!(h.atual().unwrap().numero(), 4);
    assert_eq!(
        h.selecionar(VersaoDocumento::nova(2).unwrap()),
        Err(ErroVersao::VersaoNaoObservada)
    );
}
#[test]
fn recusa_duplicada_e_documento_trocado() {
    let doc = NumeroDocumento::novo("0007").unwrap();
    assert_eq!(
        interpretar_historico(&pagina(&[1, 1], &["42", "42"]), &doc, "42", None),
        Err(ErroVersao::Html(crate::erro_html::ErroHtml::repetido(
            "versao_documento_duplicada",
            "o elemento aparece mais de uma vez"
        )))
    );
    assert_eq!(
        interpretar_historico(&pagina(&[1], &["43"]), &doc, "42", None),
        Err(ErroVersao::DocumentoDivergente)
    );
    assert_eq!(
        interpretar_historico(&pagina(&[1], &["42"]), &doc, "42", Some(2)),
        Err(ErroVersao::ListaIncompleta)
    );
}
#[test]
fn numero_zero_e_link_com_acao_repetida_sao_recusados() {
    assert_eq!(
        VersaoDocumento::nova(0),
        Err(ErroVersao::Html(crate::erro_html::ErroHtml::valor(
            "versao_documento_invalida",
            "o valor não está na forma observada"
        )))
    );
    let doc = NumeroDocumento::novo("0007").unwrap();
    let html = pagina(&[1], &["42"]).replace(
        "acao=documento_visualizar",
        "acao=documento_visualizar&amp;acao=editor_montar",
    );
    assert_eq!(
        interpretar_historico(&html, &doc, "42", None),
        Err(ErroVersao::LinkInseguro)
    );
}

#[test]
fn legendas_repetidas_nao_podem_confirmar_cobertura_de_versoes() {
    let doc = NumeroDocumento::novo("0007").unwrap();
    let pagina = pagina(&[1], &["42"]);
    let duplicada = pagina.replacen(
        "</caption>",
        "</caption><caption>Lista de Versões (1 registros):</caption>",
        1,
    );
    let extra = pagina.replacen(
            "<caption>Lista de Versões (1 registros):</caption>",
            "<caption>Outro inventário (1 registros):</caption><caption>Lista de Versões (1 registros):</caption>",
            1,
        );
    for html in [duplicada, extra] {
        assert_eq!(
            interpretar_historico(&html, &doc, "42", None),
            Err(ErroVersao::Html(crate::erro_html::ErroHtml::repetido(
                "legendas_historico_versoes_repetidas",
                "o elemento aparece mais de uma vez"
            )))
        );
    }
    assert_eq!(
        ErroVersao::Html(crate::erro_html::ErroHtml::repetido(
            "legendas_historico_versoes_repetidas",
            "o elemento aparece mais de uma vez"
        ))
        .subcodigo(),
        "legendas_historico_versoes_repetidas"
    );
}
